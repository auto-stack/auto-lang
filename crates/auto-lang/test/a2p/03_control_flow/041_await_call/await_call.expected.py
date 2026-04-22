async def fetch_data(url: str) -> str:
    return url

async def main():
    result = await fetch_data("http://example.com")
    print(result)

if __name__ == "__main__":
    asyncio.run(main())
