; indents.scm - Auto-indentation rules for Nix

; Indent after opening brackets
[
  "{"
  "["
  "("
] @indent

; Dedent on closing brackets
[
  "}"
  "]"
  ")"
] @outdent

; Indent let...in blocks
(let_expression
  "let" @indent
  "in" @outdent.always)

; Continue indent for bindings
(binding) @indent.always

; Indent function bodies
(function_expression
  ":" @indent
  body: (_) @indent.always)

; Indent if-then-else
(if_expression
  "if" @indent
  "then" @indent.branch
  "else" @indent.branch)

; Indent with expressions
(with_expression
  "with" @indent
  ";" @outdent.always)

; Indent assert expressions
(assert_expression
  "assert" @indent
  ";" @outdent.always)

; Formal parameters alignment
(formals
  "{" @indent
  "}" @outdent)

; Keep aligned for attribute paths
(attrpath) @align

; Binary operations continuation
(binary_expression
  right: _ @indent.always)

; Application continuation
(application
  argument: (_) @indent.always)

; Select continuation
(select
  "." @indent.always)

; Multi-line strings should not affect indentation
[
  (string)
  (indented_string)
] @indent.zero

; Comments are handled automatically