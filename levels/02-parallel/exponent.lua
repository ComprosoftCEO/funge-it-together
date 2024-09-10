function generateTestCase()
  local bases, powers, outputs = {}, {}, {}

  for i = 1, 8 do
    bases[i] = math.random(0, 9)

    if bases[i] <= 1 then
      powers[i] = math.random(1, 9)
    else
      powers[i] = math.random(0, math.floor(math.log(999, bases[i])))
    end
    outputs[i] = bases[i] ^ powers[i]
  end

  return powers, outputs, bases, powers
end
