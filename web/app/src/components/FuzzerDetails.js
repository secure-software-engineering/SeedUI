import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';
import Checkbox from '@mui/material/Checkbox';
import FormControlLabel from '@mui/material/FormControlLabel';
import Tooltip, { tooltipClasses } from '@mui/material/Tooltip';
import { styled } from '@mui/material/styles';

const HtmlTooltip = styled(({ className, ...props }) => (
  <Tooltip {...props} classes={{ popper: className }} />
))(({ theme }) => ({
  [`& .${tooltipClasses.tooltip}`]: {
    backgroundColor: '#f5f5f9',
    color: 'rgba(0, 0, 0, 0.87)',
    maxWidth: 220,
    border: '1px solid #dadde9',
  },
}));

function FuzzerInfoToolTip({ fuzzer_details, listener }) {
  return (
    <FormControlLabel
      control={<Checkbox style={{ transform: "scale(0.8)", padding: 1 }} id={ fuzzer_details.fuzzer_configuration_id } defaultChecked onClick={ () => listener(fuzzer_details.fuzzer_configuration_id) }/>}
      label={
        <HtmlTooltip title={
            <Stack direction="column" spacing={1}>
              <Typography variant='body2'><strong>{ fuzzer_details.fuzzer_configuration_name }</strong></Typography>
              <Divider />
              <Typography variant='body2'>#run time: <strong>{fuzzer_details.run_time.toFixed(1)} hour(s)</strong></Typography>
              <Typography variant='body2'>#total inputs: <strong>{ fuzzer_details.total_inputs }</strong></Typography>
              <Typography variant='body2'>#total initial seeds: <strong>{ fuzzer_details.total_initial_seeds }</strong></Typography>
            </Stack>
          } arrow>
          <Box component='span' style={{ cursor: 'pointer' }}><Typography variant='body2'>Run #{fuzzer_details.fuzzer_configuration_id}</Typography></Box>
        </HtmlTooltip>
      }
    />
  );
}


export default function FuzzerInfo({ setFuzzersInfo, fuzzersInfo }) {
    
  if (fuzzersInfo.size === 0) { return <div> Data loading </div> }

  const checkboxListener = function(fuzzer_configuration_id) {
    const newMap = new Map(fuzzersInfo);
    const currentItem = newMap.get(fuzzer_configuration_id);
    currentItem.checked = !currentItem.checked;
    newMap.set(fuzzer_configuration_id, currentItem);
    setFuzzersInfo(newMap);
  }

  const renderMappedEntries = () => {
      return Array.from(fuzzersInfo.entries()).map(([key, value]) => (
          <FuzzerInfoToolTip key={key} fuzzer_key={key} fuzzer_details={value} listener={checkboxListener} />
      ));
  };

  return (
      <Stack direction="row" spacing={0.5} flexWrap="wrap" justifyContent="center">
        {
          renderMappedEntries()
        }
      </Stack>
  )
}