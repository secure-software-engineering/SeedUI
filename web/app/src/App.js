import './App.css';
import { Box } from '@mui/material';
import Divider from '@mui/material/Divider';
import Chip from '@mui/material/Chip';

import { useState, useEffect } from 'react';
import { getFuzzerInfo } from './components/fetchers.js'
import FuzzerInfo from './components/FuzzerDetails';
import SUTStatistics from './components/SUTStatistics.js';
import Overview from './components/Overview.js';
import InputClusters from './components/InputClusters.js';
import ByteWiseMutations from './components/ByteWiseMutations.js';
import InputsTimeline from './components/InputsTimeline.js';
import * as d3 from 'd3';

function App() {
  const [fuzzersInfo, setFuzzersInfo] = useState(new Map());

  let { data: fuzzer_data, isLoading } = getFuzzerInfo();
  useEffect(() => {
    if (isLoading !== undefined && !isLoading) {  
      const defaultColors = d3.schemeCategory10;
      var colorIndex = 0;
      const fuzzersInfoMap = new Map();
      fuzzer_data.sort((a, b) => a.fuzzer_configuration_id - b.fuzzer_configuration_id);
      fuzzer_data.forEach(element => {
        const FInfo = {...element,
          checked: true,
          color: defaultColors[colorIndex],
        };
        fuzzersInfoMap.set(element.fuzzer_configuration_id, FInfo);
        colorIndex += 1;
      });
      setFuzzersInfo(fuzzersInfoMap);
    }
  }, [setFuzzersInfo, isLoading, fuzzer_data]);
  
  return (
    <>
      <Box sx={{ width: 0.49, float: "left", height: '100vh', borderRight: 1 }}>
        <Divider sx={{ mt: 0.1, mb: 0.1}} component="div" role="presentation" variant='fullWidth' textAlign="center" >
            <Chip sx={{ m: 0, p: 0 }} label="Source Code Coverage" size='small' variant='filled' color="success" />
        </Divider>
        <Box sx={{ overflowX: 'scroll' }}>
          <FuzzerInfo setFuzzersInfo={setFuzzersInfo} fuzzersInfo={fuzzersInfo} />
          <SUTStatistics fuzzersInfo={fuzzersInfo} />
        </Box>
        <Divider sx={{ mt: 0.5, mb: 0.5}} component="div" role="presentation" variant='fullWidth' textAlign="center" >
            <Chip sx={{ m: 0, p: 0 }} label="Fuzzer Coverage Overtime" size='small' variant='filled' color="success" />
        </Divider>
        <Overview fuzzersInfo={fuzzersInfo} />
      </Box>
      <Box sx={{ width: 0.49, float: "left", height: '100vh', overflowY: 'scroll' }}>
        <InputClusters fuzzersInfo={fuzzersInfo} />
        <InputsTimeline fuzzersInfo={fuzzersInfo} />
        <ByteWiseMutations fuzzersInfo={fuzzersInfo} />
      </Box>
    </>
  );
}

export default App;