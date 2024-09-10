function generateTestCase()
  local inputs = {}
  local abs, signs = {}, {}

  for i = 1, 8 do
    inputs[i] = math.random(-999, 999)
    abs[i] = math.abs(inputs[i])
    signs[i] = sign(inputs[i])
  end

  return inputs, abs, {}, signs
end

function sign(x)
  if x < 0 then
    return -1
  elseif x > 0 then
    return 1
  else
    return 0
  end
end
