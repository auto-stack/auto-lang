# Auto-generated from level_up.at — do not edit

extends Node

var level: int = 1

var max_health: int = 100

func level_up():
	level = level + 1
	max_health = max_health * 1.1

func _ready():
	level_up()
	print(level)
	print(max_health)
	level_up()
	print(level)
	print(max_health)
