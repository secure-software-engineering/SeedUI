import { postLineCoverageOvertime } from './fetchers.js'
import { Box } from '@mui/material';
import PropTypes from 'prop-types';
import { useState, useEffect } from 'react';
import CoverageInfo from './CoverageInfo.js'
import LinearProgress from '@mui/material/LinearProgress';

function CustomTabPanel(props) {
    const { children, value, index, ...other } = props;

    return (
        <div
            role="tabpanel"
            hidden={value !== index}
            id={`overview-tabpanel-${index}`}
            aria-labelledby={`overview-tab-${index}`}
            align="center"
            {...other}
        >
            {value === index && children}
        </div>
    );
}

CustomTabPanel.propTypes = {
    children: PropTypes.node,
    index: PropTypes.number.isRequired,
    value: PropTypes.number.isRequired,
};

function Overview ({ fuzzersInfo }) {

    const [lineCoverage, setOverviewData] = useState({});

    useEffect(() => {
        postLineCoverageOvertime({}, setOverviewData);
    }, [setOverviewData]);

    if (fuzzersInfo.size === 0 || Object.keys(lineCoverage).length === 0) return <LinearProgress sx={{ m: 5 }} />;

    return (
        <Box>
            <CoverageInfo data={lineCoverage} fuzzersInfo={fuzzersInfo} />
        </Box>
    );
}

export default Overview;