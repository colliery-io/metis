; TypeScript/TSX symbol extraction queries

; Functions
(function_declaration
  name: (identifier) @name) @function

; Classes
(class_declaration
  name: (type_identifier) @name) @class

; Interfaces (TypeScript-specific)
(interface_declaration
  name: (type_identifier) @name) @interface

; Type aliases (TypeScript-specific)
(type_alias_declaration
  name: (type_identifier) @name) @type_alias

; Enums (TypeScript-specific)
(enum_declaration
  name: (identifier) @name) @enum

; Variable/const declarations
(lexical_declaration
  (variable_declarator
    name: (identifier) @name)) @variable

; Method definitions in classes
(method_definition
  name: (property_identifier) @name) @method
