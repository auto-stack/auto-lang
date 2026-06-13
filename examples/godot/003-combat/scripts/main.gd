# Auto-generated from main.at.at — do not edit

extends Node

func get_multiplier(enemy: String) -> float:
	if enemy == "dragon":
		return 3
	elif enemy == "orc":
		return 1.5
	elif enemy == "goblin":
		return 0.5
	else:
		return 1

func calculate_damage(attack: int, enemy: String) -> int:
	var multiplier = get_multiplier(enemy)
	var damage: int = attack * multiplier
	return damage

func apply_damage(health: int, damage: int, defense: int) -> int:
	var actual: int = damage - defense
	if actual < 0:
		return health
	var remaining: int = health - actual
	if remaining < 0:
		return 0
	return remaining

func is_alive(health: int) -> bool:
	return health > 0

func _ready():
	var player_attack: int = 15
	var enemy_defense: int = 3

	var goblin_dmg = calculate_damage(player_attack, "goblin")
	print(goblin_dmg)

	var orc_dmg = calculate_damage(player_attack, "orc")
	print(orc_dmg)

	var dragon_dmg = calculate_damage(player_attack, "dragon")
	print(dragon_dmg)

	var enemy_hp: int = 80
	enemy_hp = apply_damage(enemy_hp, orc_dmg, enemy_defense)
	print(enemy_hp)

	var alive = is_alive(enemy_hp)
	print(alive)

	enemy_hp = apply_damage(enemy_hp, dragon_dmg, enemy_defense)
	print(enemy_hp)

	var dead = is_alive(enemy_hp)
	print(dead)
