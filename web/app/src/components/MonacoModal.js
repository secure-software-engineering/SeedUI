import { Editor } from "@monaco-editor/react";

import Box from '@mui/material/Box';
import Modal from '@mui/material/Modal';
import ListSubheader from '@mui/material/ListSubheader';
import Divider from '@mui/material/Divider';
import Stack from '@mui/material/Stack';
import Select from '@mui/material/Select';
import { styled } from '@mui/system';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import LinearProgress from '@mui/material/LinearProgress';
import { useState, useEffect, useRef } from "react";

import { postInitialSeedsCoverage, postInitialSeedsChildCoverage } from './fetchers.js'
import { Tooltip, Typography } from "@mui/material";

const SmallSelect = styled(Select)({
    height: '20px', // Adjust height
    fontSize: '0.8rem', // Adjust font size
    padding: '0 5px', // Adjust padding
});


export default function MonacoModal({ fileData, fuzzersInfo, modalOpen, handleClose }) {

    const editorRef = useRef(null);
    const monacoRef = useRef(null);
    const [overviewText, setOverviewText] = useState("Select a seed to view the coverage");
    const [decorationsHandle, setDecorationsHandle] = useState(null);
    const [initialSeedsCoverage, setInitialSeedsCoverage] = useState({});
    const [selectedConfiguration, setSelectedConfiguration] = useState(-1);
    const [selectedInitialSeed, setSelectedInitialSeed] = useState(-1);
    const [selectedChild, setSelectedChild] = useState(0);
    const [currentFileId, setCurrentFileId] = useState(-1);
    const [requestLoading, setRequestLoading] = useState(false);

    useEffect(() => {
        if (fuzzersInfo.size > 0 && Object.keys(fileData).length > 0) {
            postInitialSeedsCoverage({"file_id": fileData.id}, setInitialSeedsCoverage);
        }
    }, [fileData, fuzzersInfo]);

    const resetState = () => {
        setSelectedConfiguration(-1);
        setSelectedInitialSeed(-1);
        setSelectedChild(0);
        setDecorationsHandle(null);
        setOverviewText("Select a seed to view the coverage");
    };

    if (currentFileId !== fileData.id) {
        resetState();
        setCurrentFileId(fileData.id);
    }

    const renderDecorations = (selectedFuzzer, selectedIS, selectedChild) => {
        if (selectedChild === -1) {
            setRequestLoading(true);
            var coveredLinesSet = new Set();
            if (initialSeedsCoverage[selectedFuzzer][selectedIS].length > 0) {
                Array.from(initialSeedsCoverage[selectedFuzzer][selectedIS]).forEach((value, index) => {
                    if (value.hit_count > 0) {
                        coveredLinesSet.add(value.line_num);
                    }
                });
            }

            var coveredLinesDecoration = []
            coveredLinesSet.forEach(line_num => {
                coveredLinesDecoration.push({
                    range: new monacoRef.current.Range(line_num, 1, line_num, 1),
                    options: {
                        isWholeLine: true,
                        className: "parentCoveredLineDecoration",
                    },
                });
            });

            if (decorationsHandle) {
                decorationsHandle.clear();
            }
            const handle = editorRef.current.createDecorationsCollection(coveredLinesDecoration);
            setDecorationsHandle(handle);
            setOverviewText(<>Initial seed #{selectedIS} hit <Tooltip title={`Lines ${Array.from(coveredLinesSet).sort().join(', ')}`}>
                        <span className="parentCoveredLineDecoration">{coveredLinesSet.size}</span>
                    </Tooltip> lines</>);
            setRequestLoading(false);
        } else {
            setRequestLoading(true);
            postInitialSeedsChildCoverage({
                "fuzzer_configuration_id": selectedFuzzer,
                "file_id": fileData.id,
                "initial_seed_id": Number(selectedIS),
                "child_id": Number(selectedChild)
            }).then(function(childCoverage) {
                const parentCoveredLinesSet = new Set();
                if (initialSeedsCoverage[selectedFuzzer][selectedIS].length > 0) {
                    Array.from(initialSeedsCoverage[selectedFuzzer][selectedIS]).forEach((value, index) => {
                        if (value.hit_count > 0) {
                            parentCoveredLinesSet.add(value.line_num);
                        }
                    });
                }

                const childCoveredLinesSet = new Set();
                const onlyChildSet = new Set();
                const commonSet = new Set();
                Array.from(childCoverage).forEach((value, index) => {
                    if (value.hit_count > 0) {
                        childCoveredLinesSet.add(value.line_num);
                        if (parentCoveredLinesSet.has(value.line_num)) {
                            commonSet.add(value.line_num);
                        } else {
                            onlyChildSet.add(value.line_num);
                        }
                    }
                });
                
                const onlyParentSet = new Set();
                parentCoveredLinesSet.forEach((parent_line_num) => {
                    if (!childCoveredLinesSet.has(parent_line_num)) {
                        onlyParentSet.add(parent_line_num);
                    }
                });

                const coveredLinesDecoration = []
                commonSet.forEach(line_num => {
                    coveredLinesDecoration.push({
                        range: new monacoRef.current.Range(line_num, 1, line_num, 1),
                        options: {
                            isWholeLine: true,
                            className: "commonCoveredLineDecoration",
                        },
                    });
                });

                onlyParentSet.forEach(line_num => {
                    coveredLinesDecoration.push({
                        range: new monacoRef.current.Range(line_num, 1, line_num, 1),
                        options: {
                            isWholeLine: true,
                            className: "parentCoveredLineDecoration",
                        },
                    });
                });

                onlyChildSet.forEach(line_num => {
                    coveredLinesDecoration.push({
                        range: new monacoRef.current.Range(line_num, 1, line_num, 1),
                        options: {
                            isWholeLine: true,
                            className: "childCoveredLineDecoration",
                        },
                    });
                });

                if (decorationsHandle) {
                    decorationsHandle.clear();
                }
                
                const handle = editorRef.current.createDecorationsCollection(coveredLinesDecoration);
                setDecorationsHandle(handle);
                setOverviewText(<>
                    <Tooltip title={`Lines ${Array.from(commonSet).sort().join(', ')}`}>
                        <span className="commonCoveredLineDecoration">{commonSet.size}</span>
                    </Tooltip> lines are hit by both initial seed and its child. 
                    Child #{selectedChild} hit <Tooltip title={`Lines ${Array.from(onlyChildSet).sort().join(', ')}`}>
                        <span className="childCoveredLineDecoration">{onlyChildSet.size}</span>
                    </Tooltip> new lines. 
                    Initial seed #{selectedIS} alone hit <Tooltip title={`Lines ${Array.from(onlyParentSet).sort().join(', ')}`}>
                        <span className="parentCoveredLineDecoration">{onlyParentSet.size}</span>
                    </Tooltip> lines.
                </>);
                setRequestLoading(false);
            });
        }
    };

    const renderInitialSeedCategories = () => {
        const menuItems = [];
        fuzzersInfo.forEach((value, index) => {
            menuItems.push(<ListSubheader key={`monaco-model-configuration-${value.fuzzer_configuration_id}`} sx={{ display: fuzzersInfo.get(value.fuzzer_configuration_id).checked ? '' : 'none' }}>Run #{value.fuzzer_configuration_id}</ListSubheader>);
            Object.entries(value.initial_seeds_children_input_id_map).forEach((initial_seed_children_map, index_inner) => {
                menuItems.push(<MenuItem key={`parent-${value.fuzzer_configuration_id}-${initial_seed_children_map[0]}`} value={initial_seed_children_map[0]} onClick={() => {setSelectedConfiguration(value.fuzzer_configuration_id); setSelectedInitialSeed(initial_seed_children_map[0]); renderDecorations(value.fuzzer_configuration_id, initial_seed_children_map[0], -1); }}>{`seed-${initial_seed_children_map[0]}`}</MenuItem>);
            });
        });
        
        return menuItems;
    }

    const renderChildren = () => {
        const menuItems = [];
        fuzzersInfo.forEach((value, index) => {
            if (value.fuzzer_configuration_id === selectedConfiguration) {
                Object.entries(value.initial_seeds_children_input_id_map).forEach((initial_seed_children_map, index_inner) => {
                    if (initial_seed_children_map[0] === selectedInitialSeed) {
                        Array.from(initial_seed_children_map[1]).forEach((child_details, index) => {
                            menuItems.push(<MenuItem key={`monaco-child-${value.fuzzer_configuration_id}-${child_details[0]}-${child_details[1]}-${index}`} value={child_details[0]} onClick={() => {setSelectedChild(child_details[0]); renderDecorations(value.fuzzer_configuration_id, initial_seed_children_map[0], child_details[0]); }}>{`seed-${child_details[1]}`}</MenuItem>);
                        });
                    }
                });
            }
        });
        
        return menuItems;
    }

    return (
        <Modal
            open={modalOpen}
            onClose={handleClose}
            aria-labelledby="modal-modal-title"
            aria-describedby="modal-modal-description"
        >
            <Box sx={{
                    position: 'absolute',
                    top: '50%',
                    left: '50%',
                    transform: 'translate(-50%, -50%)',
                    width: '80%',
                    height: '80%',
                    bgcolor: 'background.paper',
                    boxShadow: 24,
                    p: 1,
                }}>
                <Box>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between'}}>
                        <Stack direction="row" justifyContent={'center'}>
                            <FormControl variant="standard" sx={{ m: 1, minWidth: 80 }}>
                                <InputLabel size='small'>Initial Seed</InputLabel>
                                <SmallSelect
                                    id="select-initial-seed-parent-label-id"
                                    value={selectedInitialSeed}
                                >
                                <MenuItem value={'-1-0'} onClick={() => {setSelectedConfiguration(-1); setSelectedInitialSeed(-1); }}>
                                    <em>None</em>
                                </MenuItem>
                                {
                                   renderInitialSeedCategories()
                                }
                                </SmallSelect>
                            </FormControl>
                            <FormControl variant="standard" sx={{ m: 1, minWidth: 80 }}>
                                <InputLabel htmlFor="select-initial-seed-child-label-id" size='small'>Derived Seed</InputLabel>
                                <SmallSelect
                                    id="select-initial-seed-child-label-id"
                                    value={selectedChild}
                                    disabled={selectedInitialSeed === -1}
                                >
                                <MenuItem value="0" disabled>
                                    <em>None</em>
                                </MenuItem>
                                {
                                    renderChildren()
                                }
                                </SmallSelect>
                            </FormControl>
                            <FormControl sx={{ justifyContent: 'flex-end' }}>
                                <Box component="div" margin='auto' sx={{
                                    p: 1,
                                    whiteSpace: 'normal',
                                    bgcolor: 'grey.100',
                                    color: 'grey.800',
                                    border: '1px solid',
                                    borderColor: 'grey.300',
                                    borderRadius: 2,
                                    justifyContent: 'center',
                                    minWidth: 180,
                                }}>
                                    { requestLoading && <LinearProgress /> }
                                    { !requestLoading && <Typography variant="subtitle2"> {overviewText} </Typography> }
                                </Box>
                            </FormControl>
                        </Stack>
                    </Box>
                </Box>
                <Divider />
                <Box>
                    <Editor
                        id="code-viewer"
                        height="70vh"
                        theme="light"
                        defaultLanguage="c"
                        defaultValue={fileData.content}
                        options={{ readOnly: true, wordWrap: "bounded" }}
                        onMount={(editor, monaco) => {
                            editorRef.current = editor;
                            monacoRef.current = monaco;
                        }}
                    />
                </Box>
            </Box>
        </Modal>
    );
}