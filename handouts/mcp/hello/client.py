# 前のページの server.py と同じ階層に作成
import asyncio
from fastmcp import Client

async def test_hello():
    async with Client("server.py") as client:
        result = await client.call_tool(
            "hello",
            {
                "name": "Yamada Taro"
            }
        )
        print(result.structured_content["result"])

if __name__ == "__main__":
    asyncio.run(test_hello())