; JavaScript/JSX symbol extraction queries

; Functions
(function_declaration
  name: (identifier) @name) @function

; Classes
(class_declaration
  name: (identifier) @name) @class

; Variable/const declarations
(lexical_declaration
  (variable_declarator
    name: (identifier) @name)) @variable

; Method definitions in classes
(method_definition
  name: (property_identifier) @name) @method
