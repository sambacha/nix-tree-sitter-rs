/**
 * Tree-sitter grammar for the Nix expression language
 * 
 * References:
 * - https://nixos.org/manual/nix/stable/language/
 * - https://github.com/NixOS/nix/blob/master/src/libexpr/parser.y
 */


/**
 * @file PARSER_DESCRIPTION
 * @author PARSER_AUTHOR_NAME PARSER_AUTHOR_EMAIL
 * @license PARSER_LICENSE
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: 'nix',

  externals: $ => [
    $._string_start,
    $._string_content,
    $._string_end,
    $._indented_string_start,
    $._indented_string_content,
    $._indented_string_end,
    $._interpolation_start,
    $._interpolation_end,
    $._escape_sequence,
    $._comment,
  ],

  extras: $ => [
    /\s/,
    $._comment,
  ],

  word: $ => $.identifier,

  conflicts: $ => [
    // Necessary conflicts that can't be resolved with precedence
    [$.list, $.application],
    [$.list, $.binary_expression, $.application],
    [$.binary_expression, $.application],
  ],

  precedences: $ => [
    [
      'negate',         // 12 - Unary negation (highest)
      'has',            // 11 - Has attribute (?)
      'select',         // 10 - Attribute selection
      'call',           // 9  - Function application
      'concat',         // 8  - List concatenation (++)
      'mul',            // 7  - Multiplication, division
      'add',            // 6  - Addition, subtraction
      'not',            // 5  - Logical not (!)
      'update',         // 4  - Update (//)
      'compare',        // 3  - Comparisons (<, >, <=, >=)
      'equality',       // 2  - Equality (==, !=)
      'and',            // 1  - Logical and (&&)
      'or',             // 0  - Logical or (||)
      'impl',           // -1 - Implication (->) 
      'function',       // -2 - Function definition (lowest)
    ],
  ],

  rules: {
    source_file: $ => field('expression', $._expr),

    _expr: $ => choice(
      $.identifier,
      $.integer,
      $.float,
      $.string,
      $.indented_string,
      $.path,
      $.uri,
      $.boolean,
      $.null,
      $.list,
      $.attrset,
      $.rec_attrset,
      $.let_expression,
      $.if_expression,
      $.with_expression,
      $.assert_expression,
      $.unary_expression,
      $.binary_expression,
      $.select,
      $.has_attr,
      $.application,
      $.function_expression,
      $.parenthesized_expression,
    ),

    // If expression
    if_expression: $ => prec.right('function', seq(
      'if',
      field('condition', $._expr),
      'then',
      field('consequence', $._expr),
      'else',
      field('alternative', $._expr),
    )),

    // Identifiers and keywords
    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_'-]*/,
    
    // 'or' can be used as both keyword and identifier for backward compatibility
    or_kw: $ => 'or',

    // Literals
    integer: $ => token(choice(
      /[0-9]+/,
      /0[xX][0-9a-fA-F]+/,
      /0[oO][0-7]+/,
    )),

    float: $ => token(choice(
      /[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?/,
      /[0-9]+[eE][+-]?[0-9]+/,
    )),

    boolean: $ => choice('true', 'false'),

    null: $ => 'null',

    // Strings
    string: $ => seq(
      $._string_start,
      repeat(choice(
        $._string_content,
        $._escape_sequence,
        $.string_interpolation,
      )),
      $._string_end,
    ),

    indented_string: $ => seq(
      $._indented_string_start,
      repeat(choice(
        $._indented_string_content,
        $.string_interpolation,
      )),
      $._indented_string_end,
    ),

    string_interpolation: $ => seq(
      $._interpolation_start,
      field('expression', $._expr),
      $._interpolation_end,
    ),

    // Paths
    path: $ => token(choice(
      // Absolute path
      /\/[a-zA-Z0-9._\-+][a-zA-Z0-9._\-+\/]*/,
      // Relative path
      /\.\.?\/[a-zA-Z0-9._\-+][a-zA-Z0-9._\-+\/]*/,
      // Home path
      /~\/[a-zA-Z0-9._\-+][a-zA-Z0-9._\-+\/]*/,
      // Search path
      /<[a-zA-Z0-9._\-+]+>/,
    )),

    uri: $ => /[a-zA-Z][a-zA-Z0-9+\-\.]*:[a-zA-Z0-9%\/?:@&=+$,\-_.!~*']+/,

    // Lists
    list: $ => seq(
      '[',
      field('elements', repeat($._expr)),
      ']',
    ),

    // Attribute sets
    attrset: $ => prec.dynamic(1, seq(
      '{',
      field('bindings', repeat($._binding_set)),
      '}',
    )),

    rec_attrset: $ => prec.dynamic(2, seq(
      'rec',
      '{',
      field('bindings', repeat($._binding_set)),
      '}',
    )),

    _binding_set: $ => choice(
      $.binding,
      $.inherit,
    ),

    binding: $ => seq(
      field('attrpath', $.attrpath),
      '=',
      field('expression', $._expr),
      ';',
    ),

    inherit: $ => seq(
      'inherit',
      field('from', optional(seq('(', $._expr, ')'))),
      field('attributes', repeat1(choice($.identifier, $.or_kw, $.string, $.string_interpolation))),
      ';',
    ),

    attrpath: $ => prec.left('select', sep1(
      choice(
        $.identifier,
        $.or_kw,
        $.string,
        $.string_interpolation,
        seq('${', field('expression', $._expr), '}'),
      ),
      '.',
    )),

    // Let expression
    let_expression: $ => prec.right('function', seq(
      'let',
      field('bindings', repeat1($._binding_set)),
      'in',
      field('body', $._expr),
    )),


    // With expression
    with_expression: $ => prec.right('function', seq(
      'with',
      field('expression', $._expr),
      ';',
      field('body', $._expr),
    )),

    // Assert expression
    assert_expression: $ => prec.right('function', seq(
      'assert',
      field('condition', $._expr),
      ';',
      field('body', $._expr),
    )),

    // Functions
    function_expression: $ => prec('function', seq(
      field('parameter', $._function_parameter),
      ':',
      field('body', $._expr),
    )),

    _function_parameter: $ => choice(
      $.identifier,
      $.formals,
    ),

    formals: $ => seq(
      '{',
      sep($.formal, ','),
      optional(seq(
        optional(','),
        '...',
      )),
      '}',
      optional(seq('@', field('name', $.identifier))),
    ),

    formal: $ => seq(
      field('name', $.identifier),
      optional(seq('?', field('default', $._expr))),
    ),

    // Operators
    unary_expression: $ => choice(
      prec('not', seq('!', field('argument', $._expr))),
      prec('negate', seq('-', field('argument', $._expr))),
    ),

    binary_expression: $ => choice(
      // Arithmetic
      prec.left('add', seq(
        field('left', $._expr),
        '+',
        field('right', $._expr),
      )),
      prec.left('add', seq(
        field('left', $._expr),
        '-',
        field('right', $._expr),
      )),
      prec.left('mul', seq(
        field('left', $._expr),
        '*',
        field('right', $._expr),
      )),
      prec.left('mul', seq(
        field('left', $._expr),
        '/',
        field('right', $._expr),
      )),

      // List concatenation
      prec.right('concat', seq(
        field('left', $._expr),
        '++',
        field('right', $._expr),
      )),

      // Attribute set update
      prec.right('update', seq(
        field('left', $._expr),
        '//',
        field('right', $._expr),
      )),

      // Comparison
      prec.left('compare', seq(
        field('left', $._expr),
        '<',
        field('right', $._expr),
      )),
      prec.left('compare', seq(
        field('left', $._expr),
        '>',
        field('right', $._expr),
      )),
      prec.left('compare', seq(
        field('left', $._expr),
        '<=',
        field('right', $._expr),
      )),
      prec.left('compare', seq(
        field('left', $._expr),
        '>=',
        field('right', $._expr),
      )),

      // Equality
      prec.left('equality', seq(
        field('left', $._expr),
        '==',
        field('right', $._expr),
      )),
      prec.left('equality', seq(
        field('left', $._expr),
        '!=',
        field('right', $._expr),
      )),

      // Logical
      prec.left('and', seq(
        field('left', $._expr),
        '&&',
        field('right', $._expr),
      )),
      prec.left('or', seq(
        field('left', $._expr),
        '||',
        field('right', $._expr),
      )),

      // Implication
      prec.right('impl', seq(
        field('left', $._expr),
        '->',
        field('right', $._expr),
      )),
    ),

    // Select expression (attribute access)
    select: $ => prec.left('select', seq(
      field('expression', $._expr),
      '.',
      field('attrpath', $.attrpath),
      optional(seq(
        $.or_kw,
        field('default', $._expr),
      )),
    )),

    // Has attribute
    has_attr: $ => prec('has', seq(
      field('expression', $._expr),
      '?',
      field('attrpath', $.attrpath),
    )),

    // Function application
    application: $ => prec.left('call', seq(
      field('function', $._expr),
      field('argument', $._expr),
    )),

    // Parenthesized expression
    parenthesized_expression: $ => seq(
      '(',
      field('expression', $._expr),
      ')',
    ),
  },
});

function sep(rule, separator) {
  return optional(sep1(rule, separator));
}

function sep1(rule, separator) {
  return seq(rule, repeat(seq(separator, rule)));
}
