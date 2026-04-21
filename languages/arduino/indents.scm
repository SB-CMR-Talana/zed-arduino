; Indent function bodies, class/struct bodies, namespace blocks
[
  (declaration_list)
  (field_declaration_list)
  (statement_block)
  (enumerator_list)
  (initializer_list)
] @indent

; Indent compound statements
[
  (if_statement)
  (for_statement)
  (while_statement)
  (do_statement)
  (switch_statement)
  (case_statement)
] @indent

; Indent expressions
[
  (field_expression)
  (assignment_expression)
  (binary_expression)
  (conditional_expression)
] @indent

; Indent parameter lists and argument lists
[
  (parameter_list)
  (argument_list)
] @indent

; Dedent closing braces and brackets
(_ "{" "}" @end) @indent
(_ "(" ")" @end) @indent
(_ "[" "]" @end) @indent

; Dedent break/continue in switch cases
[
  (break_statement)
  (continue_statement)
  (return_statement)
] @outdent

; Special Arduino function indentation (setup, loop)
(function_definition
  body: (compound_statement) @indent)

; Preproc indentation
(preproc_if) @indent
(preproc_ifdef) @indent
(preproc_elif) @indent
(preproc_else) @indent
