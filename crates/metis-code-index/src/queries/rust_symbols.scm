; Rust symbol extraction queries

; Structs
(struct_item
  name: (type_identifier) @name) @struct

; Enums
(enum_item
  name: (type_identifier) @name) @enum

; Functions
(function_item
  name: (identifier) @name) @function

; Traits
(trait_item
  name: (type_identifier) @name) @trait

; Impl blocks
(impl_item
  type: (type_identifier) @type_name
  trait: (type_identifier)? @trait_name) @impl

; Type aliases
(type_item
  name: (type_identifier) @name) @type_alias

; Constants
(const_item
  name: (identifier) @name) @constant

; Static variables
(static_item
  name: (identifier) @name) @static

; Modules
(mod_item
  name: (identifier) @name) @module

; Macro definitions
(macro_definition
  name: (identifier) @name) @macro
