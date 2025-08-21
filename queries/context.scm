; context.scm - Context information for Nix (breadcrumbs, outline)

; Major structural elements for outline/breadcrumbs
(let_expression
  "let" @context.begin
  bindings: (_) @context
  "in" @context.end) @context.scope

(function_expression
  parameter: (identifier) @context.parameter
  ":" @context.separator
  body: (_) @context.body) @context.function

(function_expression
  parameter: (formals) @context.parameters
  ":" @context.separator
  body: (_) @context.body) @context.function

(attrset
  "{" @context.begin
  bindings: (_) @context
  "}" @context.end) @context.object

(rec_attrset
  "rec" @context.modifier
  "{" @context.begin
  bindings: (_) @context
  "}" @context.end) @context.object.recursive

(list
  "[" @context.begin
  elements: (_) @context
  "]" @context.end) @context.array

; Named sections in attribute sets
(binding
  attrpath: (attrpath
    (identifier) @context.key)
  "=" @context.assign
  expression: (_) @context.value) @context.entry

; Module structure
((binding
  attrpath: (attrpath
    (identifier) @context.section)
  expression: (_))
  (#any-of? @context.section 
    "imports"
    "options"
    "config"
    "meta"
    "environment"
    "networking"
    "services"
    "programs"
    "users"
    "security"
    "system"
    "boot"
    "hardware"
    "virtualisation"))

; Flake structure
((binding
  attrpath: (attrpath
    (identifier) @context.flake)
  expression: (_))
  (#any-of? @context.flake
    "description"
    "inputs"
    "outputs"
    "nixConfig"))

; Package/derivation structure
((binding
  attrpath: (attrpath
    (identifier) @context.package)
  expression: (_))
  (#any-of? @context.package
    "pname"
    "version"
    "src"
    "buildInputs"
    "nativeBuildInputs"
    "propagatedBuildInputs"
    "checkInputs"
    "buildPhase"
    "installPhase"
    "meta"))

; Hierarchical paths for navigation
(select
  expression: (_) @context.base
  "."
  attrpath: (attrpath) @context.path) @context.access

; Conditional contexts
(if_expression
  "if" @context.keyword
  condition: (_) @context.condition
  "then" @context.keyword
  consequence: (_) @context.then
  "else" @context.keyword
  alternative: (_) @context.else) @context.conditional

; With scope context
(with_expression
  "with" @context.keyword
  expression: (_) @context.scope
  ";"
  body: (_) @context.body) @context.with

; Assert context
(assert_expression
  "assert" @context.keyword
  condition: (_) @context.assertion
  ";"
  body: (_) @context.body) @context.assert

; Inherit context
(inherit
  "inherit" @context.keyword
  from: (_)? @context.source
  attributes: (_) @context.inherited) @context.inherit