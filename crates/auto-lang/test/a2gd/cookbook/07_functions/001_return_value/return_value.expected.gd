# Auto-generated from return_value.at — do not edit

extends Node

var cell_size: int = 80

func convert_to_world(cell: int):
	return cell * cell_size

func _ready():
	var world_x = convert_to_world(3)
	print(world_x)
	var world_y = convert_to_world(5)
	print(world_y)
