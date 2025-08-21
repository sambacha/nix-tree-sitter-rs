; textobjects.scm - Text objects for Nix (for vim-like motions)

; Functions
(function_expression) @function.outer

(function_expression
  body: (_) @function.inner)

; Formal parameters
(formals) @parameter.outer
(formal) @parameter.inner

; Arguments in function calls
(application
  argument: (_) @parameter.inner)

; Attribute sets
[
  (attrset)
  (rec_attrset)
] @class.outer

; Inner content of attribute sets (excluding braces)
(attrset
  bindings: (_) @class.inner)

(rec_attrset
  bindings: (_) @class.inner)

; Individual bindings/attributes
(binding) @statement.outer

; Lists
(list) @container.outer

(list
  elements: (_) @container.inner)

; Let expressions
(let_expression) @block.outer

(let_expression
  bindings: (_) @block.inner.bindings
  body: (_) @block.inner.body)

; If expressions
(if_expression) @conditional.outer

(if_expression
  condition: (_) @conditional.inner.condition
  consequence: (_) @conditional.inner.then
  alternative: (_) @conditional.inner.else)

; With expressions
(with_expression) @scope.outer

(with_expression
  expression: (_) @scope.inner.context
  body: (_) @scope.inner.body)

; Assert expressions
(assert_expression) @assertion.outer

(assert_expression
  condition: (_) @assertion.inner.condition
  body: (_) @assertion.inner.body)

; Strings
[
  (string)
  (indented_string)
] @string.outer

; String content - using the whole string nodes since content tokens are external
[
  (string)
  (indented_string)
] @string.inner

; String interpolations
(string_interpolation) @embedded.outer
(string_interpolation
  expression: (_) @embedded.inner)

; Comments - handled as external tokens, not queryable

; Binary expressions
(binary_expression) @operation.outer

(binary_expression
  left: (_) @operation.inner.left
  right: (_) @operation.inner.right)

; Select expressions (attribute access)
(select) @access.outer

(select
  expression: (_) @access.inner.object
  attrpath: (_) @access.inner.field)

; Parenthesized expressions
(parenthesized_expression) @group.outer

(parenthesized_expression
  expression: (_) @group.inner)

; Attribute paths
(attrpath) @path.outer
(attrpath
  (identifier) @path.inner)

; Any top-level expression
(source_file
  expression: (_) @program.inner) @program.outer

; Values (right-hand side of bindings)
(binding
  expression: (_) @value.inner)

; Keys (left-hand side of bindings)
(binding
  attrpath: (_) @key.inner)

; Inherit statements
(inherit) @inherit.outer
(inherit
  attributes: (_) @inherit.inner)

; Numbers
[
  (integer)
  (float)
] @number.inner