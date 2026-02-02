# JaLM Grammar v0 (EBNF)

This grammar defines the **core syntax** for JaLM v0. It is intentionally small and unambiguous, and is designed to support the MVP goals: modules/imports, functions, structs/enums, pattern matching, if/for, async/task constructs, and effect annotations.

## Lexical Conventions

- **Whitespace**: spaces, tabs, and newlines separate tokens but are otherwise insignificant.
- **Comments**:
  - Line comment: `//` to end of line.
  - Block comment: `/* ... */` (nesting not required).
- **Identifiers**: `ident` starts with `[A-Za-z_]` followed by `[A-Za-z0-9_]*`.
- **Keywords** (reserved):
  `mod`, `use`, `fn`, `async`, `struct`, `enum`, `match`, `if`, `else`, `for`, `in`, `return`,
  `let`, `mut`, `true`, `false`, `scope`, `spawn`, `join`, `await`, `break`, `continue`, `as`.
- **Literals**:
  - Integer: decimal digits (`123`), with optional `_` separators.
  - Float: digits `.` digits (e.g., `1.0`).
  - String: double-quoted with escapes: `"`, `\`, `\n`, `\t`.
  - Bytes: `b"..."`.

## Grammar

### File and Modules

```
file            = module_decl? { item } EOF ;
module_decl     = "mod" ident ";" ;

item            = use_decl
                | fn_decl
                | struct_decl
                | enum_decl
                ;

use_decl        = "use" use_path ";" ;
use_path        = ident { "::" ident } [ "as" ident ] ;
```

### Types

```
type            = fn_type
                | type_atom { type_suffix } ;

fn_type         = "fn" "(" [ type_list ] ")" [ "->" type ] [ effect_set ] ;

type_list       = type { "," type } ;

type_atom       = ident
                | "(" type ")"
                | type_literal
                ;

type_literal    = "Option" "<" type ">"
                | "Result" "<" type "," type ">"
                | "Vec" "<" type ">"
                | "Map" "<" type "," type ">"
                ;

type_suffix     = "?" ;
```

### Effects

```
effect_set      = "!" "{" [ effect_list ] "}" ;
effect_list     = effect { "," effect } ;
effect          = ident ;  // expected: io, net, fs, time, rand, ffi
```

### Declarations

```
fn_decl         = [ "async" ] "fn" ident "(" [ param_list ] ")"
                  [ "->" type ] [ effect_set ] block ;

param_list      = param { "," param } ;
param           = [ "mut" ] ident ":" type ;

struct_decl     = "struct" ident "{" { struct_field } "}" ;
struct_field    = ident ":" type ";" ;

enum_decl       = "enum" ident "{" { enum_variant } "}" ;
enum_variant    = ident [ "(" [ type_list ] ")" ] ";" ;
```

### Statements

```
stmt            = let_stmt
                | expr_stmt
                | return_stmt
                | for_stmt
                | break_stmt
                | continue_stmt
                ;

let_stmt        = "let" [ "mut" ] pattern [ ":" type ] "=" expr ";" ;
return_stmt     = "return" [ expr ] ";" ;
for_stmt        = "for" pattern "in" expr block ;

break_stmt      = "break" [ expr ] ";" ;
continue_stmt   = "continue" ";" ;
expr_stmt       = expr ";" ;
```

### Patterns

```
pattern         = ident
                | "_"
                | literal
                | tuple_pattern
                | struct_pattern
                | enum_pattern
                ;

tuple_pattern   = "(" [ pattern_list ] ")" ;
pattern_list    = pattern { "," pattern } ;

struct_pattern  = ident "{" [ struct_pat_fields ] "}" ;
struct_pat_fields = struct_pat_field { "," struct_pat_field } ;
struct_pat_field  = ident [ ":" pattern ] ;

enum_pattern    = ident "(" [ pattern_list ] ")" ;
```

### Expressions (precedence climbing)

```
expr            = assign_expr ;

assign_expr     = cond_expr [ assign_op assign_expr ] ;
assign_op       = "=" | "+=" | "-=" | "*=" | "/=" | "%="
                | "&=" | "|=" | "^=" | "<<=" | ">>=" ;

cond_expr       = coalesce_expr [ "?" expr ":" cond_expr ] ;

coalesce_expr   = logic_or_expr { "??" logic_or_expr } ;

logic_or_expr   = logic_and_expr { "||" logic_and_expr } ;
logic_and_expr  = bit_or_expr { "&&" bit_or_expr } ;

bit_or_expr     = bit_xor_expr { "|" bit_xor_expr } ;
bit_xor_expr    = bit_and_expr { "^" bit_and_expr } ;
bit_and_expr    = equality_expr { "&" equality_expr } ;

equality_expr   = compare_expr { ("==" | "!=") compare_expr } ;
compare_expr    = shift_expr { ("<" | "<=" | ">" | ">=") shift_expr } ;
shift_expr      = range_expr { ("<<" | ">>") range_expr } ;

range_expr      = add_expr { (".." | "..=") add_expr } ;
add_expr        = mul_expr { ("+" | "-") mul_expr } ;
mul_expr        = unary_expr { ("*" | "/" | "%") unary_expr } ;

unary_expr      = ("!" | "-" | "~" | "await" ) unary_expr
                | cast_expr ;

cast_expr       = primary_expr { "as" type } ;

primary_expr    = literal
                | ident
                | tuple_expr
                | struct_expr
                | enum_expr
                | call_expr
                | field_expr
                | index_expr
                | if_expr
                | match_expr
                | block
                | scope_expr
                | spawn_expr
                | join_expr
                | "(" expr ")"
                ;

call_expr       = primary_expr "(" [ arg_list ] ")" ;
arg_list        = expr { "," expr } ;

field_expr      = primary_expr "." ident ;
index_expr      = primary_expr "[" expr "]" ;

if_expr         = "if" expr block [ "else" ( if_expr | block ) ] ;

match_expr      = "match" expr "{" { match_arm } "}" ;
match_arm       = pattern "=>" expr "," ;

block           = "{" { stmt } [ expr ] "}" ;

scope_expr      = "scope" block ;
spawn_expr      = "spawn" expr ;
join_expr       = "join" expr ;
```

#### Precedence Summary (high → low)

```
1. postfix        call, field, index                 (left)
2. cast           as                                 (left)
3. unary          !  -  ~  await                      (right)
4. multiplicative *  /  %                             (left)
5. additive       +  -                                (left)
6. range          ..  ..=                             (left)
7. shift          <<  >>                              (left)
8. compare        <  <=  >  >=                        (left)
9. equality       ==  !=                              (left)
10. bitwise AND   &                                   (left)
11. bitwise XOR   ^                                   (left)
12. bitwise OR    |                                   (left)
13. logical AND   &&                                  (left)
14. logical OR    ||                                  (left)
15. coalesce      ??                                  (left)
16. conditional   ?:                                  (right)
17. assignment    =  +=  -=  *=  /=  %=  &=  |=  ^=  <<=  >>=  (right)
```

### Literals and Aggregates

```
literal         = int_lit | float_lit | string_lit | bytes_lit | bool_lit ;
bool_lit        = "true" | "false" ;

tuple_expr      = "(" [ expr_list ] ")" ;
expr_list       = expr { "," expr } ;

struct_expr     = ident "{" [ field_init_list ] "}" ;
field_init_list = field_init { "," field_init } ;
field_init      = ident ":" expr ;

enum_expr       = ident "(" [ arg_list ] ")" ;
```

## Notes and Clarifications

- **Ambiguity**: `call_expr`/`field_expr`/`index_expr` are left-associative; parsers should parse postfix chains (e.g., `foo().bar[0]`).
- **Await**: only allowed in `async fn` bodies; parser accepts but type checker enforces.
- **Effects**: effect sets appear after the return type; for `fn_type` without explicit `->`, the return is `()`.
- **Enum/struct patterns**: `ident` resolution is type-directed (parser treats them as plain identifiers).
- **Match arms**: trailing comma required in v0 for simpler parsing.
- **`as` casts**: left-associative; `x as T as U` parses as `(x as T) as U`.
- **`??` vs `?:`**: `??` binds tighter than `?:`, so `a ?? b ? c : d` parses as `(a ?? b) ? c : d`, while `a ? b : c ?? d` parses as `a ? b : (c ?? d)`.
