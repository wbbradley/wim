# wim 

A console-based IDE.

## Intent

- Editor
- Pager
- Extensible

Using [wcwidth implementation for Rust](https://github.com/ridiculousfish/widecharwidth).

## Normal-mode model

Cmd ::= TextObj |
        Operator (ForcedMotion)? Motion
Operator ::= Delete | Change | Yank

ge 
## Next items

ge ge ge ge ge ge ge ge ge ge ge ge - Save - Specific editor bindings composition
  - "send-operator"

  - "send-text-object"
  this is a new line.
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
