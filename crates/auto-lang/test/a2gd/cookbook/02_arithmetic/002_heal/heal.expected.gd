# Auto-generated from heal.at — do not edit

extends Node

var health: int = 50

func heal(amount: int):
	health = health + amount

func _ready():
	heal(20)
	print(health)
	heal(30)
	print(health)
