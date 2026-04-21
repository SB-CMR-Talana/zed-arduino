(identifier) @variable
(field_identifier) @property
(namespace_identifier) @namespace

(concept_definition
    (identifier) @concept)


(call_expression
  function: (qualified_identifier
    name: (identifier) @function))

(call_expression
  (qualified_identifier
    (identifier) @function.call))

(call_expression
  (qualified_identifier
    (qualified_identifier
      (identifier) @function.call)))

(call_expression
  (qualified_identifier
    (qualified_identifier
      (qualified_identifier
        (identifier) @function.call))))

((qualified_identifier
  (qualified_identifier
    (qualified_identifier
      (qualified_identifier
        (identifier) @function.call)))) @_parent
  (#has-ancestor? @_parent call_expression))

(call_expression
  function: (identifier) @function)

(call_expression
  function: (field_expression
    field: (field_identifier) @function))

(preproc_function_def
  name: (identifier) @function.special)

(template_function
  name: (identifier) @function)

(template_method
  name: (field_identifier) @function)

(function_declarator
  declarator: (identifier) @function)

(function_declarator
  declarator: (qualified_identifier
    name: (identifier) @function))

(function_declarator
  declarator: (field_identifier) @function)

(operator_name
  (identifier)? @operator) @function

(destructor_name (identifier) @function)

; Arduino core functions - highlighted specially
((function_declarator
  declarator: (identifier) @function.special)
 (#match? @function.special "^(setup|loop)$"))

((call_expression
  function: (identifier) @function.builtin)
 (#match? @function.builtin "^(pinMode|digitalWrite|digitalRead|analogWrite|analogRead|analogReference|tone|noTone|shiftOut|shiftIn|pulseIn|pulseInLong|millis|micros|delay|delayMicroseconds|min|max|abs|constrain|map|pow|sqrt|sin|cos|tan|randomSeed|random|lowByte|highByte|bitRead|bitWrite|bitSet|bitClear|bit|attachInterrupt|detachInterrupt|interrupts|noInterrupts|Serial\\.begin|Serial\\.end|Serial\\.available|Serial\\.read|Serial\\.peek|Serial\\.flush|Serial\\.print|Serial\\.println|Serial\\.write)$"))

((namespace_identifier) @type
 (#match? @type "^[A-Z]"))

(auto) @type
(type_identifier) @type
type :(primitive_type) @type.primitive
(sized_type_specifier) @type.primitive

; Arduino-specific types
((type_identifier) @type.builtin
 (#match? @type.builtin "^(String|boolean|byte|word)$"))

(requires_clause
    constraint: (template_type
        name: (type_identifier) @concept))

(attribute
    name: (identifier) @keyword)

((identifier) @constant
 (#match? @constant "^_*[A-Z][A-Z\\d_]*$"))

; Arduino-specific constants
((identifier) @constant.builtin
 (#match? @constant.builtin "^(HIGH|LOW|INPUT|OUTPUT|INPUT_PULLUP|LED_BUILTIN|A0|A1|A2|A3|A4|A5|A6|A7|A8|A9|A10|A11|A12|A13|A14|A15|CHANGE|FALLING|RISING|DEFAULT|INTERNAL|INTERNAL1V1|INTERNAL2V56|EXTERNAL|LSBFIRST|MSBFIRST|DEC|HEX|OCT|BIN)$"))

(statement_identifier) @label
(this) @variable.special
("static_assert") @function.builtin

[
  "alignas"
  "alignof"
  "break"
  "case"
  "catch"
  "class"
  "co_await"
  "co_return"
  "co_yield"
  "concept"
  "constexpr"
  "continue"
  "decltype"
  "default"
  "delete"
  "do"
  "else"
  "enum"
  "explicit"
  "extern"
  "final"
  "for"
  "friend"
  "if"
  "inline"
  "namespace"
  "new"
  "noexcept"
  "override"
  "private"
  "protected"
  "public"
  "requires"
  "return"
  "sizeof"
  "struct"
  "switch"
  "template"
  "throw"
  "try"
  "typedef"
  "typename"
  "union"
  "using"
  "virtual"
  "while"
  (storage_class_specifier)
  (type_qualifier)
] @keyword

[
  "#define"
  "#elif"
  "#else"
  "#endif"
  "#if"
  "#ifdef"
  "#ifndef"
  "#include"
  (preproc_directive)
] @keyword

(comment) @comment

[
  (true)
  (false)
] @boolean

[
  (null)
  ("nullptr")
] @constant.builtin

(number_literal) @number

[
  (string_literal)
  (system_lib_string)
  (char_literal)
  (raw_string_literal)
] @string

[
  ","
  ":"
  "::"
  ";"
  (raw_string_delimiter)
] @punctuation.delimiter

[
  "{"
  "}"
  "("
  ")"
  "["
  "]"
] @punctuation.bracket

[
  "."
  ".*"
  "->*"
  "~"
  "-"
  "--"
  "-="
  "->"
  "="
  "!"
  "!="
  "|"
  "|="
  "||"
  "^"
  "^="
  "&"
  "&="
  "&&"
  "+"
  "++"
  "+="
  "*"
  "*="
  "/"
  "/="
  "%"
  "%="
  "<<"
  "<<="
  ">>"
  ">>="
  "<"
  "=="
  ">"
  "<="
  ">="
  "<=>"
  "||"
  "?"
] @operator

(conditional_expression ":" @operator)
(user_defined_literal (literal_suffix) @operator)
