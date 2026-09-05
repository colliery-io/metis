; Odin symbol extraction queries

; Procedures
(procedure_declaration
  (identifier) @name) @function

; Procedure groups (proc{a, b})
(overloaded_procedure_declaration
  (identifier) @name) @function

; Structs
(struct_declaration
  (identifier) @name) @struct

; Bit fields
(bit_field_declaration
  (identifier) @name) @struct

; Enums
(enum_declaration
  (identifier) @name) @enum

; Unions
(union_declaration
  (identifier) @name) @union

; Constants (also covers type aliases: `Foo :: distinct int`)
(const_declaration
  (identifier) @name) @constant

; Typed constants: `MAX: u64 : 32`
(const_type_declaration
  (identifier) @name) @constant

; Variables with an explicit type: `count: int = 0`
(var_declaration
  (identifier) @name) @variable

; Variables with an inferred type: `count := 0`
(variable_declaration
  (identifier) @name) @variable
