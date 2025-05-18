import {Component} from '@angular/core';
import {TranslatePipe} from '@ngx-translate/core';
import {RouterLink, RouterLinkActive, RouterOutlet} from '@angular/router';

@Component({
  selector: 'app-devices',
  templateUrl: './devices.component.html',
  imports: [TranslatePipe, RouterLink, RouterLinkActive, RouterOutlet],
  styleUrls: ['./devices.component.scss'],
})
export class DevicesComponent {
  // This is a placeholder component for the Devices section
  // In a real application, this would include device management functionality
}
