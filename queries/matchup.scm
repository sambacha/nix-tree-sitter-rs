; matchup.scm - Bracket matching and pair navigation for Nix

; Brackets and braces
("{" @open
 "}" @close) @punctuation.bracket

("[" @open
 "]" @close) @punctuation.bracket

("(" @open
 ")" @close) @punctuation.bracket

; Let...in pairs
(let_expression
  "let" @open
  "in" @close) @keyword.control

; If...then...else chains
(if_expression
  "if" @open
  "then" @mid
  "else" @close) @keyword.control

; String delimiters - external scanner tokens are not queryable, skipping

; Interpolation delimiters - external scanner tokens are not queryable, skipping

; Function definition
(function_expression
  parameter: (_) @open
  ":" @mid
  body: (_) @close) @function.definition

; With expression
(with_expression
  "with" @open
  expression: (_) @mid
  ";" @mid2
  body: (_) @close) @keyword.control

; Assert expression
(assert_expression
  "assert" @open
  condition: (_) @mid
  ";" @mid2
  body: (_) @close) @keyword.control