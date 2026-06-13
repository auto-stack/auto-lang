# Auto-generated from basic_spec.at — do not edit

extends Node

# Protocol: Drawable
class Drawable:
	# Abstract: must override
	func draw():
		pass
	# Abstract: must override
	func area() -> float:
		pass

func _ready():
	print("Spec defined")
