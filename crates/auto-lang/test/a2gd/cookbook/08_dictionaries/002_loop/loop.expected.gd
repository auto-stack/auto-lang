# Auto-generated from loop.at — do not edit

extends Node

var inventory = {"healing_heart": 3, "gems": 5, "sword": 1}

func display_item(name: int, count: int):
	print(name)
	print(count)

func _ready():
	for item_name in inventory:
		var item_count = inventory[item_name]
		display_item(item_name, item_count)
