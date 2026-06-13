# Auto-generated from main.at.at — do not edit

extends Node

func spawn_wave(wave_num: int, base_count: int) -> int:
	var count: int = base_count + wave_num * 2
	var hp_mult: float = 1
	hp_mult = 1 + wave_num * 0.1

	for i in range(0, count):
		var enemy_hp = 50 * hp_mult
		print(enemy_hp)

	return count

func run_game():
	var base: int = 3
	var total: int = 0
	var wave: int = 1

	while wave <= 5:
		var spawned = spawn_wave(wave, base)
		total = total + spawned
		print(total)
		wave = wave + 1

	print(total)

func _ready():
	run_game()
