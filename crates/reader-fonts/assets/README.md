# Reader fonts

Liberation Sans, Serif and Mono preserve the original Reader's Latin metrics.
The Japanese fallback is the unmodified **Noto Sans JP Regular** subset from
[Noto CJK Sans 2.004](https://github.com/notofonts/noto-cjk/tree/Sans2.004).

- Source: `Sans/SubsetOTF/JP/NotoSansJP-Regular.otf` at tag `Sans2.004`.
- Git blob: `4af556abb9cc42e585aab777b01857246a110746`.
- Size: 4,533,028 bytes.
- Copyright: © 2014–2021 Adobe (http://www.adobe.com/).
- License: SIL Open Font License 1.1, reproduced in `fonts/OFL-NotoSansJP.txt`
  from the release's root `LICENSE`.

The Japanese face supplies missing glyphs after the requested Latin face is
checked. A single regular face keeps download size bounded; FerroUI provides
synthetic bold/italic when requested. It covers Japanese text, not every CJK
language or every Unicode script. Browser fonts remain external assets and
participate in the content-addressed application/offline snapshot; desktop
builds embed the same bytes.
