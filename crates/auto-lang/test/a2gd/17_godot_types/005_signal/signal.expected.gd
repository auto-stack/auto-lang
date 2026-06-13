# Auto-generated from signal.at — do not edit

extends Area2D

signal health_changed(new_health: int)
signal game_over

func take_damage(n: int):
	health_changed.emit(n)
