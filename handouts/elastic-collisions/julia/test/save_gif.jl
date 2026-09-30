using Test
using CairoMakie
using ElasticCollisions

# Count image blocks (0x2C) of a GIF by walking its block structure.
function gif_image_blocks(bytes::Vector{UInt8})
    flags = bytes[11]
    pos = 14
    flags & 0x80 != 0 && (pos += 3 * 2^((flags & 0x07) + 1))
    skip_sub_blocks(p) = (while bytes[p] != 0; p += bytes[p] + 1; end; p + 1)
    count = 0
    while true
        b = bytes[pos]
        if b == 0x3b
            return count
        elseif b == 0x21
            pos = skip_sub_blocks(pos + 2)
        elseif b == 0x2c
            count += 1
            lflags = bytes[pos + 9]
            pos += 10
            lflags & 0x80 != 0 && (pos += 3 * 2^((lflags & 0x07) + 1))
            pos = skip_sub_blocks(pos + 1)   # skip LZW minimum code size
        else
            error("unexpected GIF block 0x$(string(b, base = 16))")
        end
    end
end

@testset "save_gif" begin
    arena = Arena(10, 6)
    world = World(random_balls(5; arena, radius = 0.4, speed = 3.0, seed = 7), arena)
    traj = simulate(world, 1.0; frame_dt = 0.1)
    mktempdir() do dir
        path = joinpath(dir, "out.gif")
        @test save_gif(path, traj; fps = 10, size = (400, 250)) == path
        bytes = read(path)
        @test String(bytes[1:6]) in ("GIF87a", "GIF89a")
        @test gif_image_blocks(bytes) >= length(traj.frames)
    end
end
