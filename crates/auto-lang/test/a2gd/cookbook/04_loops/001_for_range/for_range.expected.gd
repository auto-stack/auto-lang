# Auto-generated from for_range.at — do not edit

extends Node

func draw_rectangle(width: int, height: int):
	print(width)
	print(height)

func _ready():
	for number in range(0, 3):
		draw_rectangle(100, 100)
