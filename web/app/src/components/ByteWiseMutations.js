import { Box, Stack, Tooltip, Typography } from '@mui/material';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import Select from '@mui/material/Select';
import { Grid, styled } from '@mui/system';
import Divider from '@mui/material/Divider';
import Chip from '@mui/material/Chip';
import OutlinedInput from '@mui/material/OutlinedInput';
import LinearProgress from '@mui/material/LinearProgress';

import { useState } from 'react';
import { postCompareInputs } from './fetchers.js'

import Gradient from "javascript-color-gradient";

const SmallSelect = styled(Select)({
    height: '25px', // Adjust height
    fontSize: '0.8rem', // Adjust font size
    // padding: '0 5px', // Adjust padding
});

function toTwoDigitHex(num) {
    // Convert the number to hexadecimal and pad with leading zero if necessary
    return num.toString().padStart(2, '0').toUpperCase();
}

export default function ByteWiseMutations({ fuzzersInfo }) {

    const [comparisonData, setComparisonData] = useState(<Typography variant="span"> No initial seed is selected to display byte-wise mutation modifications. </Typography>);
    const [selectedFuzzerConfiguration, setSelectedFuzzerConfiguration] = useState(-1);
    const [selectedInitialSeedId, setSelectedInitialSeedId] = useState(-1);
    const [requestLoading, setRequestLoading] = useState(false);
    const [minMax, setMinMax] = useState([]);

    if (fuzzersInfo.size === 0) { return <LinearProgress sx={{ m: 5 }} />; }
    
    const handleFuzzerConfigurationChange = (event) => {
        setSelectedFuzzerConfiguration(event.target.value);
    };

    const handleInitialSeedChange = (event) => {
        setSelectedInitialSeedId(event.target.value);
        renderByteChanges(event.target.value);
    };

    const arrayMinMax = (arr) =>
        arr.reduce(([min, max], val) => [Math.min(min, val), Math.max(max, val)], [
        Number.POSITIVE_INFINITY,
        Number.NEGATIVE_INFINITY,
    ]);

    const renderFuzzerConfigurationMenuItems = () => {
      return Array.from(fuzzersInfo.entries()).map(([key, value]) => (
            <MenuItem value={value.fuzzer_configuration_id}>{value.fuzzer_configuration_id}</MenuItem>
        ));
    };

    const renderInitialSeeds = () => {
        if (selectedFuzzerConfiguration === -1) {
            return <MenuItem value={-1}></MenuItem>;
        }

        const menuItems = [];
        fuzzersInfo.forEach((value, index) => {
            if (value.fuzzer_configuration_id === selectedFuzzerConfiguration) {
                Object.entries(value.initial_seeds_children_input_id_map).forEach((initial_seed_children_map, index_inner) => {
                    menuItems.push(<MenuItem key={`compare-inputs-${value.fuzzer_configuration_id}-${initial_seed_children_map[0]}`} value={initial_seed_children_map[0]} >{`seed-${initial_seed_children_map[0]}`}</MenuItem>);
                });
            }
        });

        return menuItems;
    };

    const renderByteChanges = (selectedIS) => {
        if (selectedFuzzerConfiguration === -1 && Number(selectedIS) === -1) {
            setComparisonData(<Typography variant="span"> No initial seed is selected to display byte-wise mutation modifications. </Typography>);
        } else {
            setRequestLoading(true);
            postCompareInputs({
                            "fuzzer_configuration_id": selectedFuzzerConfiguration,
                            "initial_seed_id": Number(selectedIS)
                        }).then(function(comparisonData) {
                const byteWiseModifications = comparisonData.byte_modification_counts;
                if (Object.keys(byteWiseModifications).length > 0) {
                    const sortedKeys = Object.keys(byteWiseModifications).sort((a, b) => Number(a) - Number(b)).map(Number);
                    const values = Object.values(byteWiseModifications);
                    const minMax = arrayMinMax(values);
                    const colorScale = new Gradient()
                            .setColorGradient("#ffffff", "#263238")
                            .setMidpoint(Array.from(minMax).length > 1 && minMax[1] === 1 ? 2 : minMax[1])
                            .getColors();

                    setComparisonData(
                        <Grid container spacing={0.5} size={10}>
                            {
                                Array.from(sortedKeys).map((key, index) => {
                                    return <Tooltip title={`#${byteWiseModifications[key]} mutations`} placement="top-start">
                                        <span style={{ backgroundColor: colorScale[byteWiseModifications[key] > 0 ? byteWiseModifications[key] - 1 : byteWiseModifications[key]]}}>
                                            { index === 0 ? 
                                                    key > 0 ? `00-${toTwoDigitHex(key)}` : toTwoDigitHex(key) 
                                                : 
                                                    sortedKeys[index - 1] === key - 1 ? toTwoDigitHex(key) : `${toTwoDigitHex(sortedKeys[index - 1] + 1)}-${toTwoDigitHex(key)}`
                                            }
                                        </span>
                                    </Tooltip>
                                })
                            }
                        </Grid>
                    );
                    setMinMax(minMax);
                    setRequestLoading(false);
                } else {
                    setRequestLoading(false);
                    setComparisonData(<Typography variant="span"> No data available for this seed. </Typography>);
                }
            });
        }
    }

    const renderSliderValues = () => {
        if (selectedFuzzerConfiguration === -1 || selectedInitialSeedId === -1) {
            return <Box sx={{ m: 0, display: 'flex', justifyContent: 'center'}}>
                    <Typography variant="caption"># mutations</Typography>
                </Box>
        };

        return <Box sx={{ display: 'flex', justifyContent: 'space-between'}}>
                    <Typography variant="caption">{minMax[0]}</Typography>
                    <Typography variant="caption"># mutations</Typography>
                    <Typography variant="caption">{minMax[1]}</Typography>
                </Box>
    };

    return <>
        <Divider sx={{ mt: 0, mb: 1}} component="div" role="presentation" variant='fullWidth' textAlign="center" >
            <Stack direction="row" justifyContent={'center'} spacing={1}>
                <Chip sx={{ m: 0, p: 0 }} label="Byte-wise Seed Mutations" size='small' variant='filled' color="success" />
                <FormControl variant="standard" sx={{ m: 1, minWidth: 90 }}>
                    <SmallSelect
                        displayEmpty
                        input={<OutlinedInput />}
                        id="select-configuration-label-id"
                        value={selectedFuzzerConfiguration === -1 ? `Run #` : `Run #${selectedFuzzerConfiguration}`}
                        onChange={handleFuzzerConfigurationChange}
                        renderValue={(selected) => {
                            if (selected.length === 0) {
                                return <em>Configuration</em>;
                            }

                            return selected;
                        }}
                    >
                    <MenuItem disabled value="">
                        <em>Configuration</em>
                    </MenuItem>
                    {
                        renderFuzzerConfigurationMenuItems()
                    }
                    </SmallSelect>
                </FormControl>
                <FormControl variant="standard" sx={{ m: 1, minWidth: 90 }}>
                    <SmallSelect
                        displayEmpty
                        input={<OutlinedInput />}
                        id="select-initial-seed-label-id"
                        value={`seed #${selectedInitialSeedId}`}
                        onChange={handleInitialSeedChange}
                        disabled={selectedFuzzerConfiguration === -1}
                        renderValue={(selected) => {
                            if (selected.length === 0) {
                                return <em>Initial seed</em>;
                            }

                            return selected;
                        }}
                    >
                    <MenuItem disabled value="">
                        <em>Initial seed</em>
                    </MenuItem>
                    {
                        renderInitialSeeds()
                    }
                    </SmallSelect>
                </FormControl>
                <FormControl variant="standard" sx={{ m: 1, minWidth: 120 }}>
                    <input type="range" className="slider" disabled />
                    {
                        renderSliderValues()
                    }
                </FormControl>
            </Stack>
        </Divider>
        <Box sx={{ display: 'flex', flexDirection: 'column', overflow: 'auto', height: window.innerHeight / 4.5 }} >
            { requestLoading && <LinearProgress sx={{ m: 5 }} /> }
            {!requestLoading && 
                <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 1 }}>
                    { comparisonData }
                </Box>
            }
        </Box>
    </>
}