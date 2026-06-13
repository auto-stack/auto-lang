# Auto-generated from vector2.at — do not edit

extends Node

func move_towards(pos: Vector2, vel: Vector2) -> Vector2:
	return pos + vel

func tint(base: Color, alpha: float) -> Color:
	return base

func _ready():
	var p: Vector2 = Vector2(0, 0)
	var moved = move_towards(p, Vector2(1, 1))
	print(moved)
