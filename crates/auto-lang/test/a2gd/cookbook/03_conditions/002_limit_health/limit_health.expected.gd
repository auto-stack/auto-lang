# Auto-generated from limit_health.at — do not edit

extends Node

var health: int = 20

func heal(amount: int):
	health = health + amount
	if health > 80:
		health = 80

func _ready():
	heal(30)
	print(health)
	heal(50)
	print(health)
