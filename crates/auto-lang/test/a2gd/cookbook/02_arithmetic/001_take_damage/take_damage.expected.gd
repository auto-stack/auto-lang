# Auto-generated from take_damage.at — do not edit

extends Node

var health: int = 100

func take_damage(amount: int):
	health = health - amount

func _ready():
	take_damage(30)
	print(health)
	take_damage(25)
	print(health)
