function generateTestCase()
  local xs = {}
  local ys = {}

  local totalDistance = 0

  -- 1 to 7 total coordinates
  local prevCoords
  for i = 1, 2 * math.random(2, 7), 2 do
    local x = math.random(0, 99)
    local y = math.random(0, 99)

    if prevCoords == nil then
      prevCoords = { x, y }
    end

    local delta = math.abs(prevCoords[1] - x) + math.abs(prevCoords[2] - y)
    if (totalDistance + delta) > 999 then
      break -- To make sure we don't overflow 999
    end

    totalDistance = totalDistance + delta
    xs[#xs + 1] = x
    ys[#ys + 1] = y
    prevCoords = { x, y }
  end

  return xs, { totalDistance }, ys, { totalDistance }
end
