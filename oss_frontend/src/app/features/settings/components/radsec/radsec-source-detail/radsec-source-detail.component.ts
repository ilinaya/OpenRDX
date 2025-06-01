import {Component, OnInit} from '@angular/core';
import {ActivatedRoute, Router} from '@angular/router';
import {DatePipe} from '@angular/common';
import {RadSecSource} from "../../../../../shared/models/radsec.model";
import {RadsecService} from "../../../../../shared/services/radsec.service";

@Component({
  selector: 'app-radsec-source-detail',
  templateUrl: './radsec-source-detail.component.html',
  styleUrls: ['./radsec-source-detail.component.scss'],
  imports: [
    DatePipe,
  ],
})
export class RadsecSourceDetailComponent implements OnInit {
  source?: RadSecSource;
  loading = false;
  error = '';

  constructor(
    private radsecService: RadsecService,
    private router: Router,
    private route: ActivatedRoute,
  ) {
  }

  ngOnInit(): void {
    const id = this.route.snapshot.params['id'];
    if (id) {
      this.loadSource(id);
    }
  }

  loadSource(id: number): void {
    this.loading = true;
    this.radsecService.getSource(id)
      .subscribe({
        next: (source) => {
          this.source = source;
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load source. Please try again later.';
          console.error('Error loading source:', err);
          this.loading = false;
        },
      });
  }

  editSource(): void {
    if (this.source) {
      this.router.navigate(['edit'], {relativeTo: this.route});
    }
  }

  deleteSource(): void {
    if (this.source && confirm('Are you sure you want to delete this source? This action cannot be undone.')) {
      this.radsecService.deleteSource(this.source.id)
        .subscribe({
          next: () => {
            this.router.navigate(['../'], {relativeTo: this.route});
          },
          error: (err) => {
            this.error = 'Failed to delete source. Please try again later.';
            console.error('Error deleting source:', err);
          },
        });
    }
  }

}
