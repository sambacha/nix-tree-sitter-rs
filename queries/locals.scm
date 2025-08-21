; locals.scm - Variable scoping and references for Nix

; Scopes
[
  (source_file)
  (let_expression)
  (attrset)
  (rec_attrset)
  (function_expression)
  (with_expression)
] @scope

; Definitions

; Let bindings
(let_expression
  bindings: (binding
    attrpath: (attrpath
      (identifier) @definition.var)))

; Let bindings in attribute sets
(let_expression
  bindings: (inherit
    attributes: (identifier) @definition.var))

; Function parameters
(function_expression
  parameter: (identifier) @definition.parameter)

; Formal parameters
(formal
  name: (identifier) @definition.parameter)

; Formals with @ pattern
(formals
  name: (identifier) @definition.parameter)

; Attribute definitions
(binding
  attrpath: (attrpath
    (identifier) @definition.field))

; Recursive attribute set allows self-reference
(rec_attrset
  bindings: (binding
    attrpath: (attrpath
      (identifier) @definition.field)))

; Inherit statements
(inherit
  attributes: (identifier) @definition.field)

; Inherit from another scope
(inherit
  from: (_) @reference
  attributes: (identifier) @definition.field)

; References

; Variable references
(identifier) @reference

; Attribute selection
(select
  expression: (_)
  attrpath: (attrpath
    (identifier) @reference.field))

; Has attribute check
(has_attr
  expression: (_)
  attrpath: (attrpath
    (identifier) @reference.field))

; With expression brings scope into context
(with_expression
  expression: (_) @reference.scope)

; Special scoping rules

; Let bindings are available in the body
(let_expression
  bindings: (_)
  body: (_) @scope.body)

; Rec attrset allows mutual recursion
(rec_attrset) @scope.recursive

; Function body has access to parameters
(function_expression
  parameter: (_)
  body: (_) @scope.body)

; Import statements
((application
  function: (identifier) @reference.builtin
  (#eq? @reference.builtin "import"))
  argument: (_) @reference.path)

; Builtins reference
((identifier) @reference.builtin
  (#eq? @reference.builtin "builtins"))

; Self and super in recursive sets
((identifier) @reference.special
  (#any-of? @reference.special "self" "super"))

; Mark built-in variables as pre-defined
((identifier) @definition.builtin
  (#any-of? @definition.builtin
    "true"
    "false"
    "null"
    "__curPos"
    "__nixPath"
    "__storeDir"
    "__currentSystem"
    "__currentTime"
    "__nixVersion"))

; Scope management hints

; Let expression creates a new scope for its bindings
(let_expression
  "let" @scope.begin
  "in" @scope.mid)

; With expression modifies the scope
(with_expression
  "with" @scope.modifier)

; Recursive attribute sets have special scoping
(rec_attrset
  "rec" @scope.recursive_marker)