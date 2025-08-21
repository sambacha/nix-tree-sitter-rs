; folds.scm - Code folding for Nix

; Attribute sets
[
  (attrset)
  (rec_attrset)
] @fold

; Lists
(list) @fold

; Let expressions
(let_expression) @fold

; Function bodies
(function_expression) @fold

; Formal parameters
(formals) @fold

; If-then-else expressions (fold the entire expression)
(if_expression) @fold

; With expressions
(with_expression) @fold

; Assert expressions
(assert_expression) @fold

; Multi-line strings
[
  (string)
  (indented_string)
] @fold

; Comments are handled by the scanner and not directly foldable

; Parenthesized expressions (useful for complex nested expressions)
(parenthesized_expression) @fold