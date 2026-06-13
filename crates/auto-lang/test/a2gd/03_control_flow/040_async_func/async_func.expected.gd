# Auto-generated from async_func.at — do not edit

extends Node

func fetch_data() -> String:
	await get_data()
	var result = get_result()
	return result

func _ready():
	var data = fetch_data()
	print(data)
