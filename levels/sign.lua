function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 15 do
    -- 0 is very uncommon, so give it a higher chance to randomly generate
    if math.random(1, 100) > 3 then
      inputs[i] = math.random(-999, 999)
    else
      inputs[i] = 0
    end
    outputs[i] = sign(inputs[i])
  end

  return inputs, outputs
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
