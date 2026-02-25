; Go symbol extraction queries

; Functions
(function_declaration
  name: (identifier) @name) @function

; Methods
(method_declaration
  name: (field_identifier) @name) @method

; Struct types
(type_declaration
  (type_spec
    name: (type_identifier) @name
    type: (struct_type))) @struct

; Interface types
(type_declaration
  (type_spec
    name: (type_identifier) @name
    type: (interface_type))) @interface

; Constants
(const_declaration
  (const_spec
    name: (identifier) @name)) @constant

; Variables
(var_declaration
  (var_spec
    name: (identifier) @name)) @variable
