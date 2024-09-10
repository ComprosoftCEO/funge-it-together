function generateTestCase()
  local longest_sequence = math.random(2, 10)
  local sequences = { longest_sequence }

  local slots_used = longest_sequence
  while slots_used < 15 do
    local new_slot = math.random(1, math.min(15 - slots_used, longest_sequence - 1))
    sequences[#sequences + 1] = new_slot
    slots_used = slots_used + new_slot
  end

  shuffle(sequences)

  local inputs, outputs = {}, {}
  for i = 1, #sequences do
    local number = math.random(-99, 99)
    for j = 1, sequences[i] do
      inputs[#inputs + 1] = number
    end

    if sequences[i] == longest_sequence then
      outputs[1] = number
      outputs[2] = sequences[i]
    end
  end

  return inputs, outputs
end

function shuffle(list)
  for i = #list, 2, -1 do
    local j = math.random(i)
    list[i], list[j] = list[j], list[i]
  end
end
