# Auto-generated from while_move.at — do not edit

extends Node

var cell_y: int = 0

var board_size: int = 8

func _ready():
	while cell_y < board_size - 1:
		cell_y = cell_y + 1
	print(cell_y)
