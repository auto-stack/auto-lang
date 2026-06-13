# Auto-generated from main.at.at — do not edit

extends Node

class_name Stats

var name: String
var health: int
var max_health: int
var attack: int
var defense: int
var speed: int

func create_hero(name: String) -> Stats:
	return Stats(name, 100, 100, 15, 8, 12)

func create_enemy(name: String) -> Stats:
	return Stats(name, 60, 60, 10, 4, 8)

func is_alive(stats: Stats) -> bool:
	return stats.health > 0

func heal(stats: Stats, amount: int) -> Stats:
	var new_hp: int = stats.health + amount
	if new_hp > stats.max_health:
		new_hp = stats.max_health
	return Stats(stats.name, new_hp, stats.max_health, stats.attack, stats.defense, stats.speed)

func battle_round(hero: Stats, enemy: Stats) -> Stats:
	var damage: int = hero.attack - enemy.defense
	var new_hp: int = enemy.health - damage
	if new_hp < 0:
		new_hp = 0
	return Stats(enemy.name, new_hp, enemy.max_health, enemy.attack, enemy.defense, enemy.speed)

func _ready():
	var hero = create_hero("Knight")
	var enemy = create_enemy("Goblin")

	print(hero.health)
	print(enemy.health)

	enemy = battle_round(hero, enemy)
	print(enemy.health)

	hero = heal(hero, 20)
	print(hero.health)

	var alive = is_alive(enemy)
	print(alive)
