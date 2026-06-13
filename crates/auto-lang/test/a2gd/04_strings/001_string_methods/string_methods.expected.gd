# Auto-generated from string_methods.at — do not edit

extends Node

func _ready():
	var greeting: String = "  Hello, World!  "
	var trimmed = greeting.strip()
	var upper = greeting.to_upper()
	var lower = greeting.to_lower()
	var hello = greeting.begins_with("  Hello")
	var world = greeting.ends_with("World!  ")
	var replaced = greeting.replace("World", "GDScript")
	var length = len(greeting)
	print(trimmed)
	print(upper)
	print(replaced)
	print(length)
