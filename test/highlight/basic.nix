# Basic Nix syntax highlighting test
# <- comment

let
# <- keyword
  x = 42;
  #   ^ number
  # ^ variable.definition
  
  str = "hello world";
  #     ^ string
  
  interpolated = "value: ${toString x}";
  #                      ^ punctuation.special
  #                               ^ embedded
  
  path = ./relative/path.nix;
  #      ^ string.special.path
  
  url = https://example.com;
  #     ^ string.special.uri
  
  fn = x: y: x + y;
  #    ^ parameter
  #       ^ parameter
  #          ^ reference
  #              ^ reference
  
  attrs = {
    foo = "bar";
    #     ^ string
    # ^ property
    inherit x;
    #       ^ property
  };
  
  recAttrs = rec {
  #          ^ keyword
    a = 1;
    b = a + 2;
    #   ^ reference.field
  };
in
# <- keyword

if x > 0
# <- keyword
then fn x 10
# <- keyword
else assert x == 0; 0
# <- keyword
     # <- keyword

# Function calls
map (x: x + 1) [ 1 2 3 ]
# <- function.call

# Built-in functions
toString 42
# <- function.builtin

# Operators
1 + 2 * 3 / 4 - 5
# <- number
# ^ operator
#     ^ operator
#         ^ operator
#             ^ operator

true && false || null
# <- constant.builtin
#    ^ operator
#       ^ constant.builtin
#             ^ operator
#                ^ constant.builtin

# Attribute access
attrs.foo
#     ^ property
# <- reference

# With expression
with attrs; foo
# <- keyword
#           ^ reference