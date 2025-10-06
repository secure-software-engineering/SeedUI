import { useState } from "react";

import { Box } from '@mui/material';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableCell, { tableCellClasses } from '@mui/material/TableCell';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';
import { styled } from '@mui/material/styles';
import { getSUT } from './fetchers.js';
import LinearProgress from '@mui/material/LinearProgress';

import MonacoModal from './MonacoModal.js';

const StyledTableCell = styled(TableCell)(({ theme }) => ({
    [`&.${tableCellClasses.head}`]: {
        backgroundColor: theme.palette.common.black,
        color: theme.palette.common.white,
    },
    [`&.${tableCellClasses.body}`]: {
        fontSize: 12,
    },
}));

const StyledTableRow = styled(TableRow)(({ theme }) => ({
    '&:nth-of-type(odd)': {
        backgroundColor: theme.palette.action.hover,
    },
    // hide last border
    '&:last-child td, &:last-child th': {
        border: 0,
    },
}));

function SUTStatistics({ fuzzersInfo }) {
    const [modalOpen, setModalOpen] = useState(false);
    const [modalContent, setModalContent] = useState({});

    let { data: sut_file_data, isLoading } = getSUT();
    if (isLoading || fuzzersInfo.size === 0) { return <LinearProgress sx={{ m: 5 }} /> }

    const handleOpen = (row) => {
        setModalContent(row);
        setModalOpen(true);
    };

    const handleClose = () => {
        setModalOpen(false);
        setModalContent({});
    };

    const rows = []
    if (sut_file_data) { 
        Object.entries(sut_file_data).forEach((item, index) => {
            var code_lines = 0;
            item[1]['lines'].forEach((line_meta, index) => {
                if (!line_meta.is_comment) {
                    code_lines += 1;
                }
            });

            rows.push({
                "name": item[1]["name"],
                "lines": item[1]["lines"],
                "content": item[1]["content"],
                "covered": item[1]["unique_lines_covered"],
                "id": item[1]["id"],
                "code_lines": code_lines,
            })
        })
    }

    const renderTableHeaderRows = () => {
        return Array.from(fuzzersInfo.entries()).map(([key, value]) => (
        <StyledTableCell
            key={key}
            value={value}
            align="right"
            sx={{ display: fuzzersInfo.get(key).checked ? '' : 'none' }}
        >
            #{key}: Line coverage
        </StyledTableCell>
        ));
    };

    const renderTableBodyRows = (row) => {
        return Object.entries(row.covered).map(([k, v]) => {
            return <StyledTableCell align="right" key={row['id']} sx={{ display: fuzzersInfo.get(Number(k)).checked ? '' : 'none' }}>
                {v}/{row.code_lines}
            </StyledTableCell>
        });
    };

    return <Box sx={{ height: window.innerHeight / 2 }}>
        <Table size="small" m={3}>
            <TableHead>
                <TableRow>
                    <StyledTableCell>Filename</StyledTableCell>
                    {
                        renderTableHeaderRows()
                    }
                </TableRow>
            </TableHead>
            <TableBody>
                {rows.map((row) => (
                    <StyledTableRow key={row.name}>
                        <StyledTableCell component="th" scope="row" onClick={ () => handleOpen(row) }>{row.name}
                        </StyledTableCell>
                        {
                            renderTableBodyRows(row)
                        }
                        
                    </StyledTableRow >
                ))}
            </TableBody>
        </Table>

        <MonacoModal fileData={modalContent} fuzzersInfo={fuzzersInfo} modalOpen={modalOpen} handleClose={handleClose} />
    </Box>
}

export default SUTStatistics;