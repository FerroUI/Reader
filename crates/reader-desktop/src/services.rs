use crate::{
    background, files,
    operations::{FeedOperations, Ticket},
    transport,
};
use ferroui_base::{
    input::platform::ClipboardExtensions,
    threading::{Dispatcher, DispatcherPriority, DispatcherTimer},
    Ref,
};
use ferroui_controls::Window;
use reader_domain::{Action, Feed, View};
use reader_state::{AppViewModel, HostRequest, HostServices};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    time::Duration,
};

pub fn install(vm: &Rc<AppViewModel>, window: &Ref<Window>, directory: PathBuf) {
    let weak_vm = Rc::downgrade(vm);
    let weak_window = window.downgrade();
    let operations = Rc::new(FeedOperations::default());
    let save_scheduled = Rc::new(Cell::new(false));
    vm.set_services(HostServices {
        request: Rc::new(move |request| {
            let (Some(vm), Some(window)) = (weak_vm.upgrade(), weak_window.upgrade()) else {
                return;
            };
            if matches!(request, HostRequest::ScheduleSave) {
                if !save_scheduled.replace(true) {
                    let scheduled = save_scheduled.clone();
                    let weak = Rc::downgrade(&vm);
                    DispatcherTimer::run_once(
                        move || {
                            scheduled.set(false);
                            if let Some(vm) = weak.upgrade() {
                                let _ = vm.flush();
                            }
                        },
                        Duration::from_millis(350),
                        DispatcherPriority::BACKGROUND,
                    )
                    .detach();
                }
                return;
            }
            let ticket = if matches!(
                request,
                HostRequest::Refresh { .. } | HostRequest::Subscribe { .. }
            ) {
                Some(operations.begin(
                    vm.library_generation(),
                    matches!(request, HostRequest::Subscribe { .. }),
                ))
            } else {
                None
            };
            let operations = operations.clone();
            let starting_view = vm.view();
            let directory = directory.clone();
            Dispatcher::ui_thread().spawn_local(async move {
                if let Err(error) = handle(
                    &vm,
                    &window,
                    directory,
                    request,
                    operations,
                    ticket,
                    starting_view,
                )
                .await
                {
                    vm.report_error(error);
                }
            });
        }),
    });
}

async fn handle(
    vm: &Rc<AppViewModel>,
    window: &Ref<Window>,
    directory: PathBuf,
    request: HostRequest,
    operations: Rc<FeedOperations>,
    ticket: Option<Ticket>,
    starting_view: View,
) -> Result<(), String> {
    if let Some(ticket) = ticket {
        if ticket.generation != vm.library_generation() {
            operations.complete(ticket);
            return Ok(());
        }
    }
    match request {
        HostRequest::Refresh { feeds, .. } => {
            refresh(vm, feeds, operations, ticket.expect("refresh operation"))
        }
        HostRequest::Subscribe { url, .. } => {
            let ticket = ticket.expect("subscription operation");
            let now = vm.now();
            let downloaded = background::run(move || transport::feed(url, now, true)).await;
            if vm.library_generation() != ticket.generation {
                operations.complete(ticket);
                return Ok(());
            }
            let response = match downloaded {
                Ok(response) => response,
                Err(error) => {
                    finish(vm, &operations, ticket);
                    return Err(error);
                }
            };
            vm.dispatch(Action::MergeFeed {
                url: response.url.clone(),
                feed: response.feed,
            });
            let id = vm
                .state()
                .feeds
                .iter()
                .find(|f| f.url == response.url)
                .map(|f| f.id);
            if let Some(id) = id
                .filter(|_| operations.is_latest_subscription(ticket) && vm.view() == starting_view)
            {
                vm.navigate(View::Feed(id));
                vm.set_status("Subscribed.".into());
            }
            finish(vm, &operations, ticket);
        }
        HostRequest::OpenUrl { url } => launch(window, &url).await?,
        HostRequest::Email { title, url } => {
            let mut uri = url::Url::parse("mailto:").expect("mailto URI");
            uri.query_pairs_mut()
                .append_pair("subject", &title)
                .append_pair("body", &url);
            launch(window, uri.as_str()).await?;
        }
        HostRequest::Copy { contents } => copy(window, &contents).await?,
        HostRequest::Share { title, url } => {
            copy(window, &format!("{title}\n{url}")).await?;
            vm.set_status(
                "Article title and link copied. Paste them into your preferred sharing app.".into(),
            );
        }
        HostRequest::Export {
            filename, contents, ..
        } => files::export(window, filename, contents).await?,
        HostRequest::Import { format } => files::import(vm, window, format).await?,
        HostRequest::Print { title, html, .. } => {
            files::print(window, directory, title, html).await?
        }
        HostRequest::FetchImage { url } => {
            let source = url.clone();
            let bytes = background::run(move || transport::image(&source))
                .await
                .unwrap_or_default();
            let _ = reader_ui::register_image(&url, bytes);
        }
        HostRequest::FocusSearch => reader_ui::focus_search(),
        HostRequest::ScrollToArticle { id } => reader_ui::scroll_to_article(id),
        HostRequest::ScheduleSave => {
            vm.flush()?;
        }
    }
    Ok(())
}

fn finish(vm: &AppViewModel, operations: &FeedOperations, ticket: Ticket) {
    if operations.complete(ticket) && vm.library_generation() == ticket.generation {
        vm.finish_refresh();
    }
}

fn refresh(
    vm: &Rc<AppViewModel>,
    feeds: Vec<Feed>,
    operations: Rc<FeedOperations>,
    ticket: Ticket,
) {
    if feeds.is_empty() {
        finish(vm, &operations, ticket);
        return;
    }
    let remaining = Rc::new(Cell::new(feeds.len()));
    let failed = Rc::new(Cell::new(0_usize));
    let first_error = Rc::new(RefCell::new(None));
    let total = feeds.len();
    for feed in feeds {
        let vm = vm.clone();
        let operations = operations.clone();
        let remaining = remaining.clone();
        let failed = failed.clone();
        let first_error = first_error.clone();
        Dispatcher::ui_thread().spawn_local(async move {
            let url = feed.url.clone();
            let now = vm.now();
            let result = background::run(move || transport::feed(url, now, false)).await;
            let current = vm.library_generation() == ticket.generation;
            // A user may unsubscribe while a request is running. Never recreate it.
            if current && vm.state().feed(feed.id).is_some_and(|f| f.url == feed.url) {
                match result {
                    Ok(response) => vm.dispatch(Action::MergeFeed {
                        url: feed.url,
                        feed: response.feed,
                    }),
                    Err(error) => {
                        failed.set(failed.get() + 1);
                        vm.feed_error(&feed.url, &error);
                        if first_error.borrow().is_none() {
                            *first_error.borrow_mut() = Some(error);
                        }
                    }
                }
            }
            remaining.set(remaining.get() - 1);
            if remaining.get() == 0 {
                if current {
                    vm.set_status(if failed.get() == 0 {
                        format!("Updated {total} subscriptions.")
                    } else {
                        format!(
                            "Updated {} of {total} subscriptions. {} failed: {}",
                            total - failed.get(),
                            failed.get(),
                            first_error.borrow().as_deref().unwrap_or("Request failed")
                        )
                    });
                }
                finish(&vm, &operations, ticket);
            }
        });
    }
}

async fn launch(window: &Ref<Window>, target: &str) -> Result<(), String> {
    let uri = url::Url::parse(target).map_err(|e| e.to_string())?;
    if !matches!(uri.scheme(), "https" | "http" | "mailto") {
        return Err("This link type is not supported.".into());
    }
    if window.launcher().launch_uri_async(&uri).await {
        Ok(())
    } else {
        Err("Could not open this link.".into())
    }
}
async fn copy(window: &Ref<Window>, contents: &str) -> Result<(), String> {
    let clipboard = window
        .clipboard()
        .ok_or("The system clipboard is unavailable.")?;
    clipboard
        .set_text_async(Some(contents))
        .await
        .map_err(|e| e.to_string())?;
    clipboard.flush_async().await.map_err(|e| e.to_string())
}
