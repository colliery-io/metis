; Python symbol extraction queries

; Classes
(class_definition
  name: (identifier) @name) @class

; Functions
(function_definition
  name: (identifier) @name) @function

; Decorated functions/classes
(decorated_definition
  (decorator) @decorator
  definition: (_) @decorated)

; Assignments (module-level variables)
(expression_statement
  (assignment
    left: (identifier) @name)) @variable
