; injections.scm - Language injections for Nix

; Bash/Shell script injections in common derivation builders
((application
  function: (identifier) @_func
  argument: (attrset
    bindings: (binding
      attrpath: (attrpath (identifier) @_attr)
      expression: (string) @injection.content)))
  (#any-of? @_func "mkDerivation" "stdenv.mkDerivation" "runCommand" "runCommandNoCC" "writeScriptBin" "writeScript")
  (#any-of? @_attr 
    "buildPhase" 
    "checkPhase" 
    "configurePhase" 
    "installPhase" 
    "patchPhase" 
    "postBuild" 
    "postCheck" 
    "postConfigure" 
    "postInstall" 
    "postPatch" 
    "postUnpack"
    "preBuild" 
    "preCheck" 
    "preConfigure" 
    "preInstall" 
    "prePatch" 
    "preUnpack"
    "unpackPhase")
  (#set! injection.language "bash"))

; Shell script in writeShellScript/writeShellScriptBin - nested applications for curried functions
((application
  function: (application
    function: (identifier) @_func
    argument: (_))
  argument: (string) @injection.content)
  (#any-of? @_func "writeShellScript" "writeShellScriptBin" "writeScript" "writeScriptBin")
  (#set! injection.language "bash"))

; Python scripts
((application
  function: (identifier) @_func
  argument: (string) @injection.content)
  (#any-of? @_func "writePython3" "writePython3Bin" "writePython" "writePythonBin")
  (#set! injection.language "python"))

; Perl scripts
((application
  function: (identifier) @_func
  argument: (string) @injection.content)
  (#any-of? @_func "writePerl" "writePerlBin")
  (#set! injection.language "perl"))

; JSON in fromJSON/toJSON contexts
((application
  function: (identifier) @_func
  argument: (string) @injection.content)
  (#eq? @_func "fromJSON")
  (#set! injection.language "json"))

; TOML in fromTOML contexts
((application
  function: (identifier) @_func
  argument: (string) @injection.content)
  (#eq? @_func "fromTOML")
  (#set! injection.language "toml"))

; YAML in fromYAML contexts
((application
  function: (identifier) @_func
  argument: (string) @injection.content)
  (#eq? @_func "fromYAML")
  (#set! injection.language "yaml"))

; Embedded Nix in string interpolation
(string_interpolation
  expression: (_) @injection.content
  (#set! injection.language "nix"))

; Configuration files
((binding
  attrpath: (attrpath (identifier) @_attr)
  expression: (string) @injection.content)
  (#any-of? @_attr "extraConfig" "configuration" "config")
  (#set! injection.language "ini"))

; Systemd service definitions
((binding
  attrpath: (attrpath (identifier) @_attr)
  expression: (string) @injection.content)
  (#any-of? @_attr "ExecStart" "ExecStop" "ExecReload" "ExecStartPre" "ExecStartPost" "ExecStopPost")
  (#set! injection.language "bash"))

; Vim configuration
((binding
  attrpath: (attrpath (identifier) @_attr)
  expression: (string) @injection.content)
  (#any-of? @_attr "vimrc" "init.vim" "extraConfig")
  (#set! injection.language "vim"))

; SQL queries
((binding
  attrpath: (attrpath (identifier) @_attr)
  expression: (string) @injection.content)
  (#any-of? @_attr "sql" "query" "sqlQuery")
  (#set! injection.language "sql"))

; HTML content
((binding
  attrpath: (attrpath (identifier) @_attr)
  expression: (string) @injection.content)
  (#any-of? @_attr "html" "htmlContent")
  (#set! injection.language "html"))

; CSS content
((binding
  attrpath: (attrpath (identifier) @_attr)
  expression: (string) @injection.content)
  (#any-of? @_attr "css" "style" "styles")
  (#set! injection.language "css"))

; JavaScript content
((binding
  attrpath: (attrpath (identifier) @_attr)
  expression: (string) @injection.content)
  (#any-of? @_attr "javascript" "js" "script")
  (#set! injection.language "javascript"))

; Lua scripts
((application
  function: (identifier) @_func
  argument: (string) @injection.content)
  (#any-of? @_func "writeLua" "writeLuaBin")
  (#set! injection.language "lua"))

; Dockerfile content
((binding
  attrpath: (attrpath (identifier) @_attr)
  expression: (string) @injection.content)
  (#eq? @_attr "Dockerfile")
  (#set! injection.language "dockerfile"))

; Regular expressions
((application
  function: (identifier) @_func
  argument: (string) @injection.content)
  (#any-of? @_func "match" "split" "replaceStrings" "builtins.match" "builtins.split")
  (#set! injection.language "regex"))

; Comments - handled as external tokens, not queryable for injections

; C/C++ code in inline C
((binding
  attrpath: (attrpath (identifier) @_attr)
  expression: (string) @injection.content)
  (#any-of? @_attr "cCode" "cppCode" "inlineC")
  (#set! injection.language "c"))

; Rust code
((binding
  attrpath: (attrpath (identifier) @_attr)
  expression: (string) @injection.content)
  (#eq? @_attr "rustCode")
  (#set! injection.language "rust"))

; Go code
((binding
  attrpath: (attrpath (identifier) @_attr)
  expression: (string) @injection.content)
  (#eq? @_attr "goCode")
  (#set! injection.language "go"))