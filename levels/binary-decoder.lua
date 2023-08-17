function generateTestCase()
  local output = math.random(0, 999)
  local inputs = encodeBinary(output, math.random(5, 10))

  return inputs, { output }
end

function encodeBinary(x, minDigits)
  local bits = {}
  while x > 0 do
    table.insert(bits, 1, x % 2)
    x = math.floor(x / 2)
  end

  while #bits < minDigits do
    table.insert(bits, 1, 0)
  end

  return bits
end
