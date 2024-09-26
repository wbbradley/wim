# wim 

I was fiddling around building a vim-like console-based IDE. I haven't touched this
in a while...

## Intent

- Editor
- Pager
- Extensible

Using [wcwidth implementation for Rust](https://github.com/ridiculousfish/widecharwidth).

## Normal-mode model

Cmd ::= TextObj |
        Operator (ForcedMotion)? Motion
Operator ::= Delete | Change | Yank

## Next items

 - Save - Specific editor bindings composition
  - "send-operator"
  - "send-text-object"
  - "send-motion"
- Undo
- Move status-bar to editor window.
- Line numbering/gutter
- Extending text-objects and motions
- Rasterization layer (change to a set-grapheme, grpprl-spans model).
- Multi-file open & VStack|HStack
- Quickfix window
- Thinking on points of extensibility
  - Operators
  - Text Objects
  - Motions
  - Syntax highlighting
- File types
- LSP
