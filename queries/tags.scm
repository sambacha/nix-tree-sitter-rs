; tags.scm - Tag generation for Nix (for jump-to-definition)

; Function definitions
(binding
  attrpath: (attrpath
    (identifier) @name)
  expression: (function_expression) @definition.function)

; Let bindings that are functions
(let_expression
  bindings: (binding
    attrpath: (attrpath
      (identifier) @name)
    expression: (function_expression) @definition.function))

; Attribute definitions
(binding
  attrpath: (attrpath
    (identifier) @name) @definition.field)

; Package definitions (common pattern)
((binding
  attrpath: (attrpath
    (identifier) @name)
  expression: (application
    function: (identifier) @_func))
  (#any-of? @_func "mkDerivation" "buildPythonPackage" "buildRustPackage" "buildGoModule")
  (#set! "kind" "package"))

; Module definitions
((binding
  attrpath: (attrpath
    (identifier) @name)
  expression: (attrset))
  (#set! "kind" "module"))

; Service definitions
((binding
  attrpath: (attrpath
    (identifier) @name.services
    (identifier) @name)
  expression: (_))
  (#eq? @name.services "services")
  (#set! "kind" "service"))

; Option definitions
((binding
  attrpath: (attrpath
    (identifier) @name.options
    (identifier) @name)
  expression: (_))
  (#eq? @name.options "options")
  (#set! "kind" "option"))

; Overlay definitions
((function_expression
  parameter: (identifier) @name.self
  body: (function_expression
    parameter: (identifier) @name.super)) @definition.overlay
  (#eq? @name.self "self")
  (#eq? @name.super "super")
  (#set! "kind" "overlay"))

; Flake outputs
((binding
  attrpath: (attrpath
    (identifier) @name)
  expression: (_))
  (#any-of? @name 
    "packages"
    "devShells"
    "nixosConfigurations"
    "nixosModules"
    "overlays"
    "apps"
    "checks"
    "templates")
  (#set! "kind" "flake-output"))

; Top-level variable definitions
(source_file
  expression: (let_expression
    bindings: (binding
      attrpath: (attrpath
        (identifier) @name) @definition.variable)))

; Recursive attribute set members
(rec_attrset
  bindings: (binding
    attrpath: (attrpath
      (identifier) @name) @definition.field.recursive))