# Auto-generated from index_access.at — do not edit

extends Node

var inventory: Variant = ["sword", "shield", "potion", "gem", "key", "map", "ring", "scroll", "arrow"]

func use_item(index: int):
	print(inventory[index])

func _ready():
	use_item(0)
	use_item(6)
	use_item(8)
