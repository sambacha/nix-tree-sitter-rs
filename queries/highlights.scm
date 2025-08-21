; highlights.scm - Syntax highlighting for Nix

; Comments are handled by the scanner and not directly queryable

; Literals
(integer) @number
(float) @number.float

[
  (boolean)
  (null)
] @constant.builtin

; Strings
[
  (string)
  (indented_string)
] @string

; String interpolation
(string_interpolation
  expression: (_) @embedded)

; Paths and URIs
(path) @string.special.path
(uri) @string.special.uri

; Identifiers
(identifier) @variable

; Function parameters
(function_expression
  parameter: (identifier) @parameter)

(function_expression
  parameter: (formals
    (formal
      name: (identifier) @parameter)))

; Formal parameters with @ pattern
(formals
  name: (identifier) @parameter)

; Attributes in bindings
(binding
  attrpath: (attrpath
    (identifier) @property))

(inherit
  attributes: (identifier) @property)

; Keywords
[
  "let"
  "in"
  "if"
  "then"
  "else"
  "with"
  "assert"
  "rec"
  "inherit"
] @keyword

; Operators
[
  "+"
  "-"
  "*"
  "/"
  "++"
  "//"
  "=="
  "!="
  "<"
  ">"
  "<="
  ">="
  "&&"
  "||"
  "->"
  "!"
  "?"
] @operator

; Special operators
"." @punctuation.delimiter
":" @punctuation.delimiter
";" @punctuation.delimiter
"," @punctuation.delimiter
"=" @operator

; Brackets
[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

; Function calls (heuristic - identifier followed by another expression)
(application
  function: (identifier) @function.call)

(application
  function: (select
    attrpath: (attrpath
      (identifier) @function.call)))

; Built-in functions (common ones)
((identifier) @function.builtin
  (#any-of? @function.builtin
    "abort"
    "add"
    "all"
    "any"
    "attrNames"
    "attrValues"
    "baseNameOf"
    "builtins"
    "compareVersions"
    "concatLists"
    "concatMap"
    "concatStringsSep"
    "deepSeq"
    "derivation"
    "dirOf"
    "div"
    "elem"
    "elemAt"
    "fetchGit"
    "fetchTarball"
    "fetchurl"
    "filter"
    "filterSource"
    "foldl'"
    "fromJSON"
    "genList"
    "getAttr"
    "getEnv"
    "hasAttr"
    "hashFile"
    "hashString"
    "head"
    "import"
    "intersectAttrs"
    "isAttrs"
    "isBool"
    "isFloat"
    "isFunction"
    "isInt"
    "isList"
    "isNull"
    "isPath"
    "isString"
    "length"
    "lessThan"
    "listToAttrs"
    "map"
    "mapAttrs"
    "match"
    "mul"
    "parseDrvName"
    "path"
    "pathExists"
    "placeholder"
    "readDir"
    "readFile"
    "removeAttrs"
    "replaceStrings"
    "seq"
    "sort"
    "split"
    "splitVersion"
    "stringLength"
    "sub"
    "substring"
    "tail"
    "throw"
    "toFile"
    "toJSON"
    "toPath"
    "toString"
    "toXML"
    "trace"
    "tryEval"
    "typeOf"
    "zipAttrsWith"))

; Standard library functions (nixpkgs lib)
((select
  expression: (identifier) @namespace
  attrpath: (attrpath
    (identifier) @function.call))
  (#any-of? @namespace "lib" "builtins" "pkgs"))

; Attribute access
(select
  attrpath: (attrpath
    (identifier) @property))

; Has attribute
(has_attr
  attrpath: (attrpath
    (identifier) @property))

; Variable definitions
(binding
  attrpath: (attrpath
    (identifier) @variable.definition))

(formal
  name: (identifier) @parameter.definition)

; Special identifiers
((identifier) @variable.builtin
  (#any-of? @variable.builtin 
    "self"
    "super"
    "__curPos"
    "__nixPath"
    "__storeDir"
    "__currentSystem"
    "__currentTime"
    "__nixVersion"))

; Attribute set keys that are strings
(binding
  attrpath: (attrpath
    (string) @property))

(binding
  attrpath: (attrpath
    (string_interpolation) @property))

; Inherit from
(inherit
  from: (identifier) @namespace)

; Default values in formals
(formal
  default: (_) @constant.default)

; Special case for or in select
(select
  (or_kw) @keyword.operator
  default: (_) @constant.default)

; Ellipsis in formals
(formals
  "..." @punctuation.special)

; Attribute path separators
(attrpath
  "." @punctuation.delimiter)

; Dynamic attribute names
(attrpath
  "${" @punctuation.special
  "}" @punctuation.special)

; Error nodes
(ERROR) @error