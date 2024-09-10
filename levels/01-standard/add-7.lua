local TO_ADD = 7

function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 15 do
    inputs[i] = math.random(-999, 999 - TO_ADD)
    outputs[i] = inputs[i] + TO_ADD
  end

  return inputs, outputs
end
