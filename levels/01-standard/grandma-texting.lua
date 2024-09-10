local LOOKUP_TABLE = {
  [1]  = 1, [2]  = 11, [3]  = 111, -- ABC
  [4]  = 2, [5]  = 22, [6]  = 222, -- DEF
  [7]  = 3, [8]  = 33, [9]  = 333, -- GHI
  [10] = 4, [11] = 44, [12] = 444, -- JKL
  [13] = 5, [14] = 55, [15] = 555, -- MNO
  [16] = 6, [17] = 66, [18] = 666, -- PQR
  [19] = 7, [20] = 77, [21] = 777, -- STU
  [22] = 8, [23] = 88, [24] = 888, -- VWX
  [25] = 9, [26] = 99,             -- YZ
}

function generateTestCase()
  local inputs, outputs = {}, {}
  for i = 1, 15 do
    inputs[i] = math.random(1, #LOOKUP_TABLE)
    outputs[i] = LOOKUP_TABLE[inputs[i]]
  end

  return inputs, outputs
end