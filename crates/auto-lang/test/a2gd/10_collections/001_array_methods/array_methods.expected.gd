# Auto-generated from array_methods.at — do not edit

extends Node

func _ready():
	var fruits: Array[String] = ["apple", "banana", "cherry"]
	fruits.append("date")
	var last = fruits.pop()
	var count = len(fruits)
	var has_apple = "apple" in fruits
	print(count)
	print(has_apple)
