function generateTestCase()
  local input = math.random(0, 999)
  local outputs = encodeBinary(input)

  return { input }, outputs
end

function encodeBinary(x)
  local bits = {}
  while x > 0 do
    table.insert(bits, 1, x % 2)
    x = math.floor(x / 2)
  end
  return bits
end
